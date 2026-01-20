mod config;

use clap::{Parser, Subcommand};
use config::ArenaConfig;

#[derive(Parser)]
#[command(name = "bot-arena")]
#[command(about = "Chess bot comparison tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a match between two bots
    Match {
        /// White bot name
        white: String,
        /// Black bot name
        black: String,
        /// Number of games to play
        #[arg(short, long, default_value = "10")]
        games: u32,
    },
}

fn main() {
    let cli = Cli::parse();
    let config = ArenaConfig::load().unwrap_or_default();

    match cli.command {
        Commands::Match {
            white,
            black,
            games,
        } => {
            println!("Running {} games: {} vs {}", games, white, black);
            if let Ok(bot) = config.get_bot(&white) {
                println!("White bot path: {:?}", bot.path);
            }
        }
    }
}
