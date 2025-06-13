use std::collections::HashMap;
use std::net::SocketAddr;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PeerStatus {
    NeverTried,
    ConnectedRecently,
    Unreachable,
    Banned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub last_seen: Option<u64>,         
    pub last_connected: Option<u64>,    
    pub status: PeerStatus,
    pub services: Option<u64>,      
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PeerDatabase {
    pub peers: HashMap<SocketAddr, PeerInfo>,
}

impl PeerDatabase {
    pub fn load_from_file(path: &str) -> Self {
        if let Ok(mut file) = File::open(path) {
            let mut data = String::new();
            if file.read_to_string(&mut data).is_ok() {
                if let Ok(db) = serde_json::from_str(&data) {
                    return db;
                }
            }
        }
        PeerDatabase::default()
    }

    pub fn save_to_file(&self, path: &str) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Ok(mut file) = OpenOptions::new().write(true).create(true).truncate(true).open(path) {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }

    pub fn register_peer(&mut self, addr: SocketAddr, services: Option<u64>) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let entry = self.peers.entry(addr).or_insert(PeerInfo {
            address: addr,
            last_seen: Some(now),
            last_connected: None,
            status: PeerStatus::NeverTried,
            services,
        });
        entry.last_seen = Some(now);
        // Só atualiza services se for Some
        if let Some(s) = services {
            entry.services = Some(s);
        }
    }

    pub fn _update_status(&mut self, addr: SocketAddr, status: PeerStatus) {
        if let Some(peer) = self.peers.get_mut(&addr) {
            peer.status = status.clone();
            if status == PeerStatus::ConnectedRecently {
                peer.last_connected = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs());
            }
        }
    }
}