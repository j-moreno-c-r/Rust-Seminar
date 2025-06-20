use tokio::net::TcpStream;
use std::net::SocketAddr;

pub async fn crawl_peer(addr: SocketAddr) {
    match TcpStream::connect(addr).await {
        Ok(mut stream) => {
            println!("✅ Conectado a {}", addr);
        }
        Err(e) => {
            println!("❌ Falha ao conectar em {}: {}", addr, e);
        }
    }
}

pub async fn run_crawlers(peers: Vec<SocketAddr>) {
    let mut handles = Vec::new();
    for addr in peers {
        let handle = tokio::spawn(crawl_peer(addr));
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
}