use tokio::sync::mpsc::Sender;
use std::net::SocketAddr;
use crate::p2p::database::{PeerStatus, PeerDatabase};
use crate::p2p::log::{LogLevel, Event, log, LogMessage};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use crate::p2p::messageheader::MessageHeader;

pub async fn run_crawlers_with_log(
    peers: Vec<SocketAddr>,
    db_tx: Sender<DbCommand>,
    log_tx: std::sync::mpsc::Sender<LogMessage>,
    crawl_connected: Arc<Mutex<HashSet<SocketAddr>>>,
) {
    let mut handles = Vec::new();
    for addr in peers {
        let db_tx = db_tx.clone();
        let log_tx = log_tx.clone();
        let crawl_connected = crawl_connected.clone();
        let handle = tokio::spawn(async move {
            log(&log_tx, LogLevel::Info, Event::Custom(format!("Iniciando crawl em {}", addr)));
            crate::p2p::multhread::crawl_peer_with_log(addr, db_tx, log_tx, crawl_connected).await;
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
    crawl_connected: Arc<Mutex<HashSet<SocketAddr>>>,
) {
    match tokio::net::TcpStream::connect(addr).await {
        Ok(mut _stream) => {
            log(&log_tx, LogLevel::Info, Event::Connected(addr));
            let _ = db_tx.send(DbCommand::UpdatePeerStatus(addr, PeerStatus::ConnectedRecently)).await;
            crawl_connected.lock().unwrap().insert(addr);

            let header = MessageHeader::new("getaddr", &[]);
            let mut _message = header.to_bytes();
            if let Err(e) = _stream.try_write(&_message) {
                log(&log_tx, LogLevel::Info, Event::Custom(format!("Falha ao enviar getaddr para {}: {}", addr, e)));
            } else {
                log(&log_tx, LogLevel::Info, Event::Custom(format!("Enviado getaddr para {}", addr)));
            }
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
    _RegisterPeer(SocketAddr, Option<u64>),
}


pub async fn multhread_db(
    mut db: PeerDatabase,
    mut rx: tokio::sync::mpsc::Receiver<DbCommand>,
    db_path: &'static str,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            DbCommand::UpdatePeerStatus(addr, status) => db._update_status(addr, status),
            DbCommand::_RegisterPeer(addr, services) => db.register_peer(addr, services),
        }
        db.save_to_file(db_path);
    }

}