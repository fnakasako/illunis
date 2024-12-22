use anyhow::Result;
use clap::{Parser, Subcommand};
use sap::{
    content::{ActionType, ConditionType, Content, Rule},
    LocalProcessor,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "sap")]
#[command(about = "Sovereign Attention Protocol CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new content filtering rule
    AddRule {
        /// Unique rule identifier
        #[arg(short, long)]
        id: String,
        
        /// Condition type (keyword, regex, ml)
        #[arg(short, long)]
        condition_type: String,
        
        /// Condition value
        #[arg(short, long)]
        value: String,
        
        /// Action type (filter, modify, flag)
        #[arg(short, long)]
        action: String,
        
        /// Action parameters as JSON string
        #[arg(short, long)]
        params: Option<String>,
    },

    /// List all content filtering rules
    ListRules,

    /// View attention metrics
    Metrics {
        /// Specific content ID to view metrics for
        #[arg(short, long)]
        id: Option<String>,
        
        /// Show top N most interacted content
        #[arg(short, long, default_value = "10")]
        top: usize,
    },

    /// Process a piece of content
    Process {
        /// Content identifier
        #[arg(short, long)]
        id: String,
        
        /// Content text
        #[arg(short, long)]
        text: String,
        
        /// View duration in seconds
        #[arg(short, long, default_value = "0")]
        duration: i64,
    },

    /// Clean up old metrics data
    Cleanup {
        /// Keep data from last N days
        #[arg(short, long, default_value = "30")]
        days: i64,
    },

    /// Export metrics to JSON file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Import metrics from JSON file
    Import {
        /// Input file path
        #[arg(short, long)]
        input: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Initialize LocalProcessor with default database path
    let db_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".sap")
        .join("metrics.db");
    
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let database_url = format!("sqlite:{}", db_path.display());
    
    let processor = LocalProcessor::new(&database_url).await?;

    match cli.command {
        Commands::AddRule {
            id,
            condition_type,
            value,
            action,
            params,
        } => {
            let condition = match condition_type.as_str() {
                "keyword" => ConditionType::Keyword(value),
                "regex" => ConditionType::Regex(value),
                "ml" => ConditionType::MachineLearning {
                    model_id: value,
                    threshold: 0.5,
                },
                _ => anyhow::bail!("Invalid condition type"),
            };

            let action_type = match action.as_str() {
                "filter" => ActionType::Filter,
                "modify" => ActionType::Modify {
                    transform: params.unwrap_or_else(|| "{content}".to_string()),
                },
                "flag" => {
                    let flags = params
                        .map(|p| serde_json::from_str(&p))
                        .transpose()?
                        .unwrap_or_else(|| vec!["flagged".to_string()]);
                    ActionType::Flag { flags }
                }
                _ => anyhow::bail!("Invalid action type"),
            };

            let rule = Rule {
                id,
                condition,
                action: action_type,
            };

            processor.add_rule(rule).await?;
            info!("Rule added successfully");
        }

        Commands::ListRules => {
            let rules = processor.get_rules().await?;
            if rules.is_empty() {
                info!("No rules found");
            } else {
                for rule in rules {
                    println!("Rule: {}", rule.id);
                    println!("  Condition: {:?}", rule.condition);
                    println!("  Action: {:?}", rule.action);
                    println!();
                }
            }
        }

        Commands::Metrics { id, top } => {
            if let Some(content_id) = id {
                if let Some(metrics) = processor.get_metrics(&content_id).await? {
                    println!("Metrics for content {}", content_id);
                    println!("  Duration: {}ms", metrics.total_duration);
                    println!("  Interactions: {}", metrics.interactions);
                    println!("  Last interaction: {}", metrics.last_interaction);
                } else {
                    info!("No metrics found for this content ID");
                }
            } else {
                let metrics = processor.get_all_metrics().await?;
                println!("Top {} most interacted content:", top);
                for metric in metrics.iter().take(top) {
                    println!("Content: {}", metric.content_id);
                    println!("  Duration: {}ms", metric.total_duration);
                    println!("  Interactions: {}", metric.interactions);
                    println!();
                }
            }
        }

        Commands::Process { id, text, duration } => {
            let content = Content {
                id,
                text,
                view_duration: duration * 1000, // convert to milliseconds
                metadata: HashMap::new(),
                flags: vec![],
            };

            match processor.process_content(content).await? {
                Some(processed) => {
                    println!("Content processed successfully:");
                    println!("  ID: {}", processed.id);
                    println!("  Text: {}", processed.text);
                    if !processed.flags.is_empty() {
                        println!("  Flags: {:?}", processed.flags);
                    }
                }
                None => info!("Content was filtered out by rules"),
            }
        }

        Commands::Cleanup { days } => {
            processor.cleanup(days).await?;
            info!("Cleaned up metrics older than {} days", days);
        }

        Commands::Export { output } => {
            let store = processor.get_store();
            store.export_metrics(output).await?;
            info!("Metrics exported successfully");
        }

        Commands::Import { input } => {
            let store = processor.get_store();
            store.import_metrics(input).await?;
            info!("Metrics imported successfully");
        }
    }

    Ok(())
}
