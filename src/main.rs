use clap::{command, Parser, Subcommand};
use solvalmon::{monitor::monitor, sentry::{config::SentryConfig, sentry}};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands
}

#[derive(Subcommand)]
enum Commands {
    Monitor {
        config: String
    },
    Sentry {
        config: String
    }
}

#[tokio::main]
pub async fn main() {
    let args = Cli::parse();
    match args.cmd {
        Commands::Monitor { config } => {
            monitor::run(config).await;
        }
        Commands::Sentry { config } => {
            let config = SentryConfig::new(&config);
            sentry::run(config).await;
        }
    }
}