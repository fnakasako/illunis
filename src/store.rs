use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use crate::{attention::Metrics, content::Rule};
use chrono::{DateTime, Utc};
use std::path::Path;

/// Database operations for persistent storage
pub struct DataStore {
    pool: SqlitePool,
}

impl DataStore {
    /// Create a new DataStore instance
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize database schema
    pub async fn initialize(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                content_id TEXT PRIMARY KEY,
                total_duration INTEGER NOT NULL,
                interactions INTEGER NOT NULL,
                last_interaction INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY,
                condition TEXT NOT NULL,
                action TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_metrics_last_interaction 
            ON metrics(last_interaction);
            
            CREATE INDEX IF NOT EXISTS idx_rules_updated 
            ON rules(updated_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create database if it doesn't exist
    pub async fn create_database(database_url: &str) -> Result<()> {
        if !Sqlite::database_exists(database_url).await? {
            Sqlite::create_database(database_url).await?;
        }
        Ok(())
    }

    /// Save metrics to database
    pub async fn save_metrics(&self, content_id: &str, metrics: &Metrics) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO metrics 
            (content_id, total_duration, interactions, last_interaction, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(content_id)
        .bind(metrics.total_duration)
        .bind(metrics.interactions)
        .bind(metrics.last_interaction.timestamp())
        .bind(metrics.created_at.timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get metrics for specific content
    pub async fn get_metrics(&self, content_id: &str) -> Result<Option<Metrics>> {
        let record = sqlx::query!(
            r#"
            SELECT * FROM metrics WHERE content_id = ?
            "#,
            content_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| Metrics {
            content_id: r.content_id,
            total_duration: r.total_duration,
            interactions: r.interactions,
            last_interaction: DateTime::from_timestamp(r.last_interaction, 0)
                .unwrap_or_else(|| Utc::now()),
            created_at: DateTime::from_timestamp(r.created_at, 0)
                .unwrap_or_else(|| Utc::now()),
        }))
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> Result<Vec<Metrics>> {
        let records = sqlx::query!(
            r#"
            SELECT * FROM metrics 
            ORDER BY last_interaction DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| Metrics {
                content_id: r.content_id,
                total_duration: r.total_duration,
                interactions: r.interactions,
                last_interaction: DateTime::from_timestamp(r.last_interaction, 0)
                    .unwrap_or_else(|| Utc::now()),
                created_at: DateTime::from_timestamp(r.created_at, 0)
                    .unwrap_or_else(|| Utc::now()),
            })
            .collect())
    }

    /// Save rule to database
    pub async fn save_rule(&self, rule: &Rule) -> Result<()> {
        let now = Utc::now().timestamp();
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO rules 
            (id, condition, action, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&rule.id)
        .bind(serde_json::to_string(&rule.condition)?)
        .bind(serde_json::to_string(&rule.action)?)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get rule by ID
    pub async fn get_rule(&self, rule_id: &str) -> Result<Option<Rule>> {
        let record = sqlx::query!(
            r#"
            SELECT * FROM rules WHERE id = ?
            "#,
            rule_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|r| Rule {
            id: r.id,
            condition: serde_json::from_str(&r.condition).unwrap(),
            action: serde_json::from_str(&r.action).unwrap(),
        }))
    }

    /// Get all rules
    pub async fn get_all_rules(&self) -> Result<Vec<Rule>> {
        let records = sqlx::query!(
            r#"
            SELECT * FROM rules 
            ORDER BY updated_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|r| Rule {
                id: r.id,
                condition: serde_json::from_str(&r.condition).unwrap(),
                action: serde_json::from_str(&r.action).unwrap(),
            })
            .collect())
    }

    /// Clean up old metrics
    pub async fn cleanup(&self, days_to_keep: i64) -> Result<()> {
        let cutoff = Utc::now().timestamp() - (days_to_keep * 24 * 60 * 60);
        
        sqlx::query!(
            r#"
            DELETE FROM metrics 
            WHERE last_interaction < ?
            "#,
            cutoff
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Export metrics to JSON file
    pub async fn export_metrics<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let metrics = self.get_all_metrics().await?;
        let json = serde_json::to_string_pretty(&metrics)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Import metrics from JSON file
    pub async fn import_metrics<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = tokio::fs::read_to_string(path).await?;
        let metrics: Vec<Metrics> = serde_json::from_str(&json)?;
        
        for metric in metrics {
            self.save_metrics(&metric.content_id, &metric).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_test_db() -> Result<(SqlitePool, DataStore)> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        DataStore::create_database(&database_url).await?;
        let pool = SqlitePool::connect(&database_url).await?;
        let store = DataStore::new(pool.clone());
        store.initialize().await?;
        
        Ok((pool, store))
    }

    #[tokio::test]
    async fn test_metrics_crud() -> Result<()> {
        let (_pool, store) = setup_test_db().await?;
        
        let metrics = Metrics {
            content_id: "test".to_string(),
            total_duration: 1000,
            interactions: 1,
            last_interaction: Utc::now(),
            created_at: Utc::now(),
        };

        // Create
        store.save_metrics(&metrics.content_id, &metrics).await?;

        // Read
        let saved = store.get_metrics(&metrics.content_id).await?.unwrap();
        assert_eq!(saved.total_duration, metrics.total_duration);
        assert_eq!(saved.interactions, metrics.interactions);

        Ok(())
    }

    #[tokio::test]
    async fn test_rules_crud() -> Result<()> {
        let (_pool, store) = setup_test_db().await?;
        
        use crate::content::{ConditionType, ActionType};

        let rule = Rule {
            id: "test".to_string(),
            condition: ConditionType::Keyword("test".to_string()),
            action: ActionType::Filter,
        };

        // Create
        store.save_rule(&rule).await?;

        // Read
        let saved = store.get_rule(&rule.id).await?.unwrap();
        assert_eq!(saved.id, rule.id);

        Ok(())
    }
}
