use tokio::net::TcpStream;
use std::net::SocketAddr;

pub async fn crawl_peer(addr: SocketAddr) -> Result<(), String> {
    match TcpStream::connect(addr).await {
        Ok(_stream) => {
            println!("✅ Conectado a {}", addr);
            Ok(())
        }
        Err(e) => {
            println!("❌ Falha ao conectar em {}: {}", addr, e);
            Err(format!("Falha ao conectar em {}: {}", addr, e))
        }
    }
}

pub async fn run_crawlers(peers: Vec<SocketAddr>) {
    let mut handles = Vec::new();
    for addr in peers {
        let handle = tokio::spawn(async move {
            if let Err(e) = crawl_peer(addr).await {
                println!("Erro no crawler: {}", e);
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
}