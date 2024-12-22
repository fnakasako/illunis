use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Content to be processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    /// Unique identifier for the content
    pub id: String,
    /// Main text content
    pub text: String,
    /// View duration in milliseconds
    pub view_duration: i64,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
    /// Optional flags from filtering
    pub flags: Vec<String>,
}

/// Condition types for filtering rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    /// Simple keyword matching
    Keyword(String),
    /// Regular expression pattern
    Regex(String),
    /// Machine learning model inference
    #[serde(rename = "ml")]
    MachineLearning {
        model_id: String,
        threshold: f32,
    },
}

/// Action types for filtering rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Remove content completely
    Filter,
    /// Modify content text
    Modify {
        transform: String,
    },
    /// Add flags to content
    Flag {
        flags: Vec<String>,
    },
}

/// Rule for content filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Condition to evaluate
    pub condition: ConditionType,
    /// Action to take when condition matches
    pub action: ActionType,
}

/// Content filter implementing rule-based filtering
pub struct ContentFilter {
    /// Active filtering rules
    rules: HashMap<String, Rule>,
    /// Cached regular expressions
    regex_cache: Arc<RwLock<HashMap<String, Regex>>>,
}

impl ContentFilter {
    /// Create a new ContentFilter instance
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            regex_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new filtering rule
    pub fn add_rule(&mut self, rule: Rule) -> Result<()> {
        // Pre-compile regex if needed
        if let ConditionType::Regex(pattern) = &rule.condition {
            let mut cache = self.regex_cache.try_write()?;
            if !cache.contains_key(pattern) {
                let regex = Regex::new(pattern)?;
                cache.insert(pattern.clone(), regex);
            }
        }
        
        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Process content through filtering rules
    pub async fn process_content(&self, content: &Content) -> Result<Option<Content>> {
        for rule in self.rules.values() {
            if self.evaluate_condition(&rule.condition, content).await? {
                return self.execute_action(&rule.action, content).await;
            }
        }
        Ok(Some(content.clone()))
    }

    /// Evaluate a condition against content
    async fn evaluate_condition(&self, condition: &ConditionType, content: &Content) -> Result<bool> {
        match condition {
            ConditionType::Keyword(keyword) => {
                Ok(content.text.to_lowercase().contains(&keyword.to_lowercase()))
            }
            ConditionType::Regex(pattern) => {
                let cache = self.regex_cache.read().await;
                if let Some(regex) = cache.get(pattern) {
                    Ok(regex.is_match(&content.text))
                } else {
                    // Fallback compilation if not in cache
                    let regex = Regex::new(pattern)?;
                    Ok(regex.is_match(&content.text))
                }
            }
            ConditionType::MachineLearning { model_id, threshold } => {
                // Placeholder for ML inference
                // In a real implementation, this would load and use the model
                Ok(false)
            }
        }
    }

    /// Execute an action on content
    async fn execute_action(&self, action: &ActionType, content: &Content) -> Result<Option<Content>> {
        match action {
            ActionType::Filter => Ok(None),
            ActionType::Modify { transform } => {
                let mut new_content = content.clone();
                // Simple text transformation for now
                // Could be extended to support more complex transformations
                new_content.text = transform.replace("{content}", &content.text);
                Ok(Some(new_content))
            }
            ActionType::Flag { flags } => {
                let mut new_content = content.clone();
                new_content.flags.extend(flags.iter().cloned());
                Ok(Some(new_content))
            }
        }
    }

    /// Get all active rules
    pub fn get_rules(&self) -> Vec<Rule> {
        self.rules.values().cloned().collect()
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> Option<Rule> {
        self.rules.remove(rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_keyword_filtering() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut filter = ContentFilter::new();
            
            filter.add_rule(Rule {
                id: "no-ads".to_string(),
                condition: ConditionType::Keyword("sponsored".to_string()),
                action: ActionType::Filter,
            }).unwrap();

            let content = Content {
                id: "test".to_string(),
                text: "This is a sponsored post".to_string(),
                view_duration: 0,
                metadata: HashMap::new(),
                flags: vec![],
            };

            assert!(filter.process_content(&content).await.unwrap().is_none());
        });
    }

    #[test]
    fn test_regex_filtering() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut filter = ContentFilter::new();
            
            filter.add_rule(Rule {
                id: "no-urls".to_string(),
                condition: ConditionType::Regex(r"https?://\S+".to_string()),
                action: ActionType::Flag { flags: vec!["contains-url".to_string()] },
            }).unwrap();

            let content = Content {
                id: "test".to_string(),
                text: "Check this link: https://example.com".to_string(),
                view_duration: 0,
                metadata: HashMap::new(),
                flags: vec![],
            };

            let processed = filter.process_content(&content).await.unwrap().unwrap();
            assert!(processed.flags.contains(&"contains-url".to_string()));
        });
    }

    #[test]
    fn test_content_modification() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut filter = ContentFilter::new();
            
            filter.add_rule(Rule {
                id: "clean-text".to_string(),
                condition: ConditionType::Keyword("bad".to_string()),
                action: ActionType::Modify {
                    transform: "Content filtered for inappropriate language".to_string(),
                },
            }).unwrap();

            let content = Content {
                id: "test".to_string(),
                text: "This is bad content".to_string(),
                view_duration: 0,
                metadata: HashMap::new(),
                flags: vec![],
            };

            let processed = filter.process_content(&content).await.unwrap().unwrap();
            assert_eq!(processed.text, "Content filtered for inappropriate language");
        });
    }
}
