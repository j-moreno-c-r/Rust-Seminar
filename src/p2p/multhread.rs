use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use std::net::SocketAddr;
use crate::p2p::database::{PeerStatus, PeerDatabase};
use crate::p2p::log::{LogLevel, Event, log, LogMessage};


pub async fn run_crawlers_with_log(
    peers: Vec<SocketAddr>,
    db_tx: Sender<DbCommand>,
    log_tx: std::sync::mpsc::Sender<LogMessage>,
) {
    let mut handles = Vec::new();
    for addr in peers {
        let db_tx = db_tx.clone();
        let log_tx = log_tx.clone();
        let handle = tokio::spawn(async move {
            log(&log_tx, LogLevel::Info, Event::Custom(format!("Iniciando crawl em {}", addr)));
            crate::p2p::multhread::crawl_peer_with_log(addr, db_tx, log_tx).await;
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
}

pub async fn crawl_peer_with_log(
    addr: SocketAddr,
    db_tx: Sender<DbCommand>,
    log_tx: std::sync::mpsc::Sender<LogMessage>,
) {
    match tokio::net::TcpStream::connect(addr).await {
        Ok(_stream) => {
            log(&log_tx, LogLevel::Info, Event::Connected(addr));
            let _ = db_tx.send(DbCommand::UpdatePeerStatus(addr, PeerStatus::ConnectedRecently)).await;
        }
        Err(e) => {
            log(&log_tx, LogLevel::Warn, Event::FailedConnection(addr, e.to_string()));
            let _ = db_tx.send(DbCommand::UpdatePeerStatus(addr, PeerStatus::Unreachable)).await;
        }
    }
}


#[derive(Debug)]
pub enum DbCommand {
    UpdatePeerStatus(SocketAddr, PeerStatus),
    RegisterPeer(SocketAddr, Option<u64>),
}

pub async fn crawl_peer(
    addr: SocketAddr,
    db_tx: Sender<DbCommand>,
) {
    match TcpStream::connect(addr).await {
        Ok(_stream) => {
            println!("✅ Conectado a {}", addr);
            let _ = db_tx.send(DbCommand::UpdatePeerStatus(addr, PeerStatus::ConnectedRecently)).await;
        }
        Err(e) => {
            println!("❌ Falha ao conectar em {}: {}", addr, e);
            let _ = db_tx.send(DbCommand::UpdatePeerStatus(addr, PeerStatus::Unreachable)).await;
        }
    }
}

pub async fn database_task(
    mut db: PeerDatabase,
    mut rx: tokio::sync::mpsc::Receiver<DbCommand>,
    db_path: &'static str,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            DbCommand::UpdatePeerStatus(addr, status) => db._update_status(addr, status),
            DbCommand::RegisterPeer(addr, services) => db.register_peer(addr, services),
        }
        db.save_to_file(db_path);
    }
}

pub async fn run_crawlers(peers: Vec<SocketAddr>, db_tx: Sender<DbCommand>) {
    let mut handles = Vec::new();
    for addr in peers {
        let db_tx = db_tx.clone();
        let handle = tokio::spawn(async move {
            crawl_peer(addr, db_tx).await;
        });
        handles.push(handle);
    }
    for handle in handles {
        let _ = handle.await;
    }
}