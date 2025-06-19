use clap::{Parser};
use std::path::PathBuf;

/// A Bitcoin P2P client for connecting to and exploring the Bitcoin network
#[derive(Parser, Debug)]
#[command(name = "bitcoin-client")]
#[command(about = "A Bitcoin P2P client for network exploration and crawling")]
#[command(long_about = None)]
#[command(version)]
pub struct Cli {
    /// Bitcoin node hostname to connect to
    #[arg(long, default_value = "seed.bitcoin.sipa.be")]
    pub host: String,

    /// Verbosity level for logging (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub verbosity: String,

    /// Bitcoin node port to connect to
    #[arg(long, default_value_t = 8333)]
    pub port: u16,

    /// Number of concurrent crawling threads (future feature)
    #[arg(long, default_value_t = 1)]
    pub threads: usize,

    /// Path to log file for output (future feature)
    #[arg(long)]
    pub logfile: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Maximum number of messages to process before exiting
    #[arg(long, default_value_t = 500000)]
    pub max_messages: u64,

    /// Connection timeout in seconds
    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    /// Enable address discovery mode (send getaddr)
    #[arg(long)]
    pub discover_peers: bool,

    /// Protocol version to advertise
    #[arg(long, default_value_t = 70015)]
    pub protocol_version: u32,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse() -> Self {
        Parser::parse()
    }

    /// Get the socket address as a string
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Print the current configuration
    pub fn print_config(&self) {
        println!("🔧 Configuration:");
        println!("   Host: {}", self.host);
        println!("   Port: {}", self.port);
        println!("   Socket: {}", self.socket_addr());
        println!("   Threads: {} (stubbed)", self.threads);
        println!("   Max messages: {}", self.max_messages);
        println!("   Timeout: {}s", self.timeout);
        println!("   Protocol version: {}", self.protocol_version);
        println!("   Verbose: {}", self.verbose);
        println!("   Discover peers: {}", self.discover_peers);
        
        if let Some(ref logfile) = self.logfile {
            println!("   Log file: {} (stubbed)", logfile.display());
        } else {
            println!("   Log file: None");
        }
        println!();
    }
}

