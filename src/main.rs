mod p2p;
mod cli;
mod tests;
mod interactive;

use interactive::InteractiveCli;
use p2p::log::{Logger, LogLevel};

fn main() -> std::io::Result<()> {
    let config = cli::Cli::parse();

    let min_level = LogLevel::from_str(&config.verbosity).unwrap_or(LogLevel::Trace);

    let (log_tx, log_rx) = std::sync::mpsc::channel();

    let _logger_handle = Logger::spawn(min_level, log_rx);

    let mut cli = InteractiveCli::new_with_logger(config, log_tx);
    cli.run()
}