mod week_two;

use week_two::BitcoinClient;

fn main() -> std::io::Result<()> {
    let mut client = BitcoinClient::new();
    client.run()
}