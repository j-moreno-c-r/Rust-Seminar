use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::UdpSocket;
use crate::p2p::database::PeerDatabase;
use rand::seq::IteratorRandom;
use crate::p2p::log::{LogLevel};


const MAX_PEERS: usize = 10;
const DNS_PORT: u16 = 1053; 

pub async fn run_dns_server(
    peer_db: Arc<RwLock<PeerDatabase>>,
    domain: &str,
    log_tx: std::sync::mpsc::Sender<crate::p2p::log::LogMessage>,
) -> std::io::Result<()> {
    let addr = format!("0.0.0.0:{}", DNS_PORT);
    let socket = UdpSocket::bind(&addr).await?;
    crate::p2p::log::log(&log_tx, LogLevel::Info, crate::p2p::log::Event::Custom(format!("DNS server listening on {}", addr)));

    let mut buf = [0u8; 512];
    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let req = &buf[..len];

        if let Some((txid, is_a_query, qname)) = parse_dns_query(req) {
            if is_a_query && qname == domain {
                let peers = {
                    let db = peer_db.read().unwrap();
                    db.peers
                        .values()
                        .filter(|p| p.status == crate::p2p::database::PeerStatus::ConnectedRecently)
                        .map(|p| p.address)
                        .choose_multiple(&mut rand::rng(), MAX_PEERS)
                };
                let response = build_dns_response(req, txid, &qname, &peers);
                let _ = socket.send_to(&response, src).await;
                crate::p2p::log::log(
                    &log_tx,
                    LogLevel::Info,
                    crate::p2p::log::Event::Custom(format!("Respondido para {} com {} peers", src, peers.len()))
                );
            } else {
                let response = build_dns_notimpl(req, txid);
                let _ = socket.send_to(&response, src).await;
                crate::p2p::log::log(
                    &log_tx,
                    LogLevel::Warn,
                    crate::p2p::log::Event::Custom(format!("Consulta não suportada de {}", src))
                );
            }
        } else {
            crate::p2p::log::log(
                &log_tx,
                LogLevel::Warn,
                crate::p2p::log::Event::Custom(format!("Consulta DNS malformada de {}", src))
            );
        }
    }
}



fn parse_dns_query(req: &[u8]) -> Option<(u16, bool, String)> {
    if req.len() < 12 {
        return None;
    }
    let txid = u16::from_be_bytes([req[0], req[1]]);
    let flags = u16::from_be_bytes([req[2], req[3]]);
    let qdcount = u16::from_be_bytes([req[4], req[5]]);
    if flags & 0x8000 != 0 || qdcount == 0 {
        return None;
    }

    let mut idx = 12;
    let mut labels = Vec::new();
    while idx < req.len() && req[idx] != 0 {
        let len = req[idx] as usize;
        idx += 1;
        if idx + len > req.len() {
            return None;
        }
        labels.push(String::from_utf8_lossy(&req[idx..idx+len]).to_string());
        idx += len;
    }
    if idx >= req.len() {
        return None;
    }
    idx += 1; 

    if idx + 4 > req.len() {
        return None;
    }
    let qtype = u16::from_be_bytes([req[idx], req[idx+1]]);
    let qclass = u16::from_be_bytes([req[idx+2], req[idx+3]]);
    let qname = labels.join(".");

    let is_a_query = qtype == 1 && qclass == 1;
    Some((txid, is_a_query, qname))
}

fn build_dns_response(req: &[u8], txid: u16, _qname: &str, peers: &[SocketAddr]) -> Vec<u8> {
    let mut resp = Vec::new();

    resp.extend(&txid.to_be_bytes()); 
    resp.extend(&[0x81, 0x80]); 
    resp.extend(&[0x00, 0x01]); 
    resp.extend(&(peers.len() as u16).to_be_bytes()); // ANCOUNT
    resp.extend(&[0x00, 0x00]); // NSCOUNT
    resp.extend(&[0x00, 0x00]); // ARCOUNT

 
    let mut idx = 12;
    while idx < req.len() {
        if req[idx] == 0 {
            idx += 5; 
            break;
        }
        idx += 1 + req[idx] as usize;
    }
    resp.extend(&req[12..idx]);

    for peer in peers {
        resp.extend(&[0xC0, 0x0C]);
        resp.extend(&[0x00, 0x01]); // TYPE = A
        resp.extend(&[0x00, 0x01]); // CLASS = IN
        resp.extend(&[0x00, 0x00, 0x00, 0x3C]); // TTL = 60s
        resp.extend(&[0x00, 0x04]); // RDLENGTH = 4

        let ip = match peer {
            SocketAddr::V4(addr) => addr.ip().octets(),
            SocketAddr::V6(_) => [0, 0, 0, 0], // Ignora IPv6
        };
        resp.extend(&ip);
    }
    resp
}

fn build_dns_notimpl(req: &[u8], txid: u16) -> Vec<u8> {
    let mut resp = Vec::new();
    resp.extend(&txid.to_be_bytes());
    resp.extend(&[0x81, 0x84]); // RCODE=4
    resp.extend(&[0x00, 0x01]); // QDCOUNT = 1
    resp.extend(&[0x00, 0x00]); // ANCOUNT
    resp.extend(&[0x00, 0x00]); // NSCOUNT
    resp.extend(&[0x00, 0x00]); // ARCOUNT

    let mut idx = 12;
    while idx < req.len() {
        if req[idx] == 0 {
            idx += 5;
            break;
        }
        idx += 1 + req[idx] as usize;
    }
    resp.extend(&req[12..idx]);
    resp
}