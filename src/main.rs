use std::path::PathBuf;

use clap::{Parser, Subcommand};

use serpentine_backend::{self, config::ConfigProvider};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run {
        /// Path to config file
        #[clap(short, long)]
        config: PathBuf,
    },
    /// Config operations
    #[clap(subcommand)]
    Config(ConfigCommand),
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Validate JSON config file
    ValidateJson {
        /// Path to config file to validate
        config: PathBuf,
    },
    /// Print default JSON config file
    PrintDefaultJson,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let args = Args::parse();

    match args.command {
        Command::Run { config: c } => {
            let config_provider = match serpentine_backend::config::setup_config_provider(c).await {
                Ok(provider) => provider,
                Err(e) => {
                    tracing::error!("Failed to set up config provider: {}", e);
                    std::process::exit(1);
                }
            };

            serpentine_backend::start_server(match config_provider.get().await {
                Ok(config) => config,
                Err(e) => {
                    tracing::error!("Failed to get config: {}", e);
                    std::process::exit(1);
                }
            })
            .await;
        }
        Command::Config(config_command) => match config_command {
            ConfigCommand::ValidateJson { config: p } => {
                if let Err(e) = serpentine_backend::config::builtin::json::validate_config(p).await
                {
                    tracing::error!("Config validation failed: {}", e);
                    std::process::exit(1);
                } else {
                    tracing::info!("Config validation succeeded");
                    std::process::exit(0);
                }
            }
            ConfigCommand::PrintDefaultJson => {
                let default_config = serpentine_backend::config::Config::default();
                match serde_json::to_string_pretty(&default_config) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        tracing::error!("Failed to serialize default config: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
    }
}
