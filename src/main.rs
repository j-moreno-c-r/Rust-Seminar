mod p2p_client;

use p2p_client::BitcoinClient;

fn main() -> std::io::Result<()> {
    let mut client = BitcoinClient::new();
    client.run()
}