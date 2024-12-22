use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::SqlitePool;

pub mod attention;
pub mod content;
pub mod store;
pub mod federation;

/// Core processor for the Sovereign Attention Protocol
pub struct LocalProcessor {
    attention_tracker: Arc<Mutex<attention::AttentionTracker>>,
    content_filter: Arc<Mutex<content::ContentFilter>>,
    data_store: Arc<store::DataStore>,
}

impl LocalProcessor {
    /// Create a new LocalProcessor instance
    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(db_path).await?;
        let data_store = Arc::new(store::DataStore::new(pool));
        
        // Initialize database schema
        data_store.initialize().await?;

        Ok(Self {
            attention_tracker: Arc::new(Mutex::new(attention::AttentionTracker::new())),
            content_filter: Arc::new(Mutex::new(content::ContentFilter::new())),
            data_store: data_store,
        })
    }

    /// Process content through filters and track attention
    pub async fn process_content(&self, content: content::Content) -> anyhow::Result<Option<content::Content>> {
        // Apply content filtering
        let filtered = {
            let filter = self.content_filter.lock().await;
            filter.process_content(&content).await?
        };

        // If content wasn't filtered out, track attention
        if let Some(processed) = filtered {
            let mut tracker = self.attention_tracker.lock().await;
            tracker.track_focus(&processed.id, processed.view_duration);
            
            // Persist metrics
            if let Some(metrics) = tracker.get_focus_metrics(&processed.id) {
                self.data_store.save_metrics(&processed.id, metrics).await?;
            }

            Ok(Some(processed))
        } else {
            Ok(None)
        }
    }

    /// Add a new content filtering rule
    pub async fn add_rule(&self, rule: content::Rule) -> anyhow::Result<()> {
        // Add rule to filter
        {
            let mut filter = self.content_filter.lock().await;
            filter.add_rule(rule.clone())?;
        }

        // Persist rule
        self.data_store.save_rule(&rule).await?;
        Ok(())
    }

    /// Get metrics for specific content
    pub async fn get_metrics(&self, content_id: &str) -> anyhow::Result<Option<attention::Metrics>> {
        self.data_store.get_metrics(content_id).await
    }

    /// Get all attention metrics
    pub async fn get_all_metrics(&self) -> anyhow::Result<Vec<attention::Metrics>> {
        self.data_store.get_all_metrics().await
    }

    /// Get all active rules
    pub async fn get_rules(&self) -> anyhow::Result<Vec<content::Rule>> {
        self.data_store.get_all_rules().await
    }

    /// Clean up old metrics data
    pub async fn cleanup(&self, days_to_keep: i64) -> anyhow::Result<()> {
        self.data_store.cleanup(days_to_keep).await
    }
}
