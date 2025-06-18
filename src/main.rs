mod p2p;
mod cli;
mod interactive;

use interactive::InteractiveCli;

fn main() -> std::io::Result<()> {
    let mut cli = InteractiveCli::new();
    cli.run()
}