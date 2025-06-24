use tokio::net::TcpStream;
use std::net::SocketAddr;
use futures::FutureExt; 

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
            // Protege cada task contra panic
            let res = std::panic::AssertUnwindSafe(crawl_peer(addr))
                .catch_unwind()
                .await;
            match res {
                Ok(Ok(_)) => {} // Sucesso
                Ok(Err(e)) => println!("Erro no crawler: {}", e),
                Err(_) => println!("❌ Panic na task do crawler para {}", addr),
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        if let Err(e) = handle.await {
            println!("❌ Task do crawler panicked ou foi cancelada: {:?}", e);
        }
    }
}
