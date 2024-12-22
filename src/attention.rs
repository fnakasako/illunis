use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metrics for content interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Unique identifier for the content
    pub content_id: String,
    /// Total duration of attention in milliseconds
    pub total_duration: i64,
    /// Number of interactions with the content
    pub interactions: i32,
    /// Timestamp of last interaction
    pub last_interaction: DateTime<Utc>,
    /// When the metrics were first created
    pub created_at: DateTime<Utc>,
}

/// Tracks user attention metrics for content
pub struct AttentionTracker {
    /// Map of content IDs to their metrics
    metrics: HashMap<String, Metrics>,
    /// When the tracker was initialized
    start_time: DateTime<Utc>,
}

impl AttentionTracker {
    /// Create a new AttentionTracker instance
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            start_time: Utc::now(),
        }
    }

    /// Track focus time for specific content
    pub fn track_focus(&mut self, content_id: &str, duration: i64) {
        let now = Utc::now();
        
        self.metrics
            .entry(content_id.to_string())
            .and_modify(|m| {
                m.total_duration += duration;
                m.interactions += 1;
                m.last_interaction = now;
            })
            .or_insert(Metrics {
                content_id: content_id.to_string(),
                total_duration: duration,
                interactions: 1,
                last_interaction: now,
                created_at: now,
            });
    }

    /// Get metrics for specific content
    pub fn get_focus_metrics(&self, content_id: &str) -> Option<Metrics> {
        self.metrics.get(content_id).cloned()
    }

    /// Get all tracked metrics
    pub fn get_all_metrics(&self) -> Vec<Metrics> {
        self.metrics.values().cloned().collect()
    }

    /// Get total tracked duration across all content
    pub fn get_total_duration(&self) -> i64 {
        self.metrics.values().map(|m| m.total_duration).sum()
    }

    /// Get average interaction duration
    pub fn get_average_duration(&self) -> Option<f64> {
        if self.metrics.is_empty() {
            None
        } else {
            let total: i64 = self.metrics.values().map(|m| m.total_duration).sum();
            let count: i32 = self.metrics.values().map(|m| m.interactions).sum();
            Some(total as f64 / count as f64)
        }
    }

    /// Get most interacted content
    pub fn get_most_interacted(&self, limit: usize) -> Vec<Metrics> {
        let mut metrics: Vec<_> = self.metrics.values().cloned().collect();
        metrics.sort_by(|a, b| b.interactions.cmp(&a.interactions));
        metrics.truncate(limit);
        metrics
    }

    /// Get recently interacted content
    pub fn get_recent_interactions(&self, limit: usize) -> Vec<Metrics> {
        let mut metrics: Vec<_> = self.metrics.values().cloned().collect();
        metrics.sort_by(|a, b| b.last_interaction.cmp(&a.last_interaction));
        metrics.truncate(limit);
        metrics
    }

    /// Calculate attention distribution percentages
    pub fn get_attention_distribution(&self) -> HashMap<String, f64> {
        let total_duration = self.get_total_duration() as f64;
        if total_duration == 0.0 {
            return HashMap::new();
        }

        self.metrics
            .iter()
            .map(|(id, m)| {
                let percentage = (m.total_duration as f64 / total_duration) * 100.0;
                (id.clone(), percentage)
            })
            .collect()
    }

    /// Get tracker uptime in seconds
    pub fn get_uptime(&self) -> i64 {
        (Utc::now() - self.start_time).num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_track_focus() {
        let mut tracker = AttentionTracker::new();
        tracker.track_focus("test-content", 1000);
        
        let metrics = tracker.get_focus_metrics("test-content").unwrap();
        assert_eq!(metrics.total_duration, 1000);
        assert_eq!(metrics.interactions, 1);
    }

    #[test]
    fn test_multiple_interactions() {
        let mut tracker = AttentionTracker::new();
        
        tracker.track_focus("test-content", 1000);
        sleep(Duration::from_millis(10));
        tracker.track_focus("test-content", 2000);

        let metrics = tracker.get_focus_metrics("test-content").unwrap();
        assert_eq!(metrics.total_duration, 3000);
        assert_eq!(metrics.interactions, 2);
    }

    #[test]
    fn test_attention_distribution() {
        let mut tracker = AttentionTracker::new();
        
        tracker.track_focus("content-1", 6000); // 60%
        tracker.track_focus("content-2", 4000); // 40%

        let distribution = tracker.get_attention_distribution();
        
        assert!((distribution["content-1"] - 60.0).abs() < f64::EPSILON);
        assert!((distribution["content-2"] - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_most_interacted() {
        let mut tracker = AttentionTracker::new();
        
        tracker.track_focus("content-1", 1000);
        tracker.track_focus("content-2", 1000);
        tracker.track_focus("content-2", 1000);
        
        let most_interacted = tracker.get_most_interacted(1);
        assert_eq!(most_interacted[0].content_id, "content-2");
        assert_eq!(most_interacted[0].interactions, 2);
    }
}
