mod p2p;
mod cli;

use p2p::p2p_client::BitcoinClient;
use cli::Cli;

fn main() -> std::io::Result<()> {
    let config = Cli::parse();
    
    config.print_config();
    
    let mut client = BitcoinClient::new();
    

    println!("🚀 Starting Bitcoin P2P Client with custom configuration");
    println!("📡 Connecting to: {}", config.socket_addr());
    
    if config.verbose {
        println!("🔍 Verbose mode enabled");
    }
    
    if config.discover_peers {
        println!("🌐 Peer discovery mode enabled");
    }
    
    if config.threads > 1 {
        println!("⚠️  Multi-threading not yet implemented (threads: {})", config.threads);
    }
    
    if config.logfile.is_some() {
        println!("⚠️  Log file output not yet implemented");
    }
    
    client.run()
}