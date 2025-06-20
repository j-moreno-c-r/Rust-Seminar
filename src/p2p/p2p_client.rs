use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::time::{Duration};
use std::io::{Write, Read, ErrorKind};
use crate::p2p::messageheader::MessageHeader;
use crate::p2p::utils::{sha256d, MAGIC, parse_inv_message,parse_addr_message ,build_version_payload,build_getdata_payload};
use crate::p2p::database::{PeerDatabase};
use crate::p2p::log::{LogLevel, Event, log, LogMessage};
use std::sync::mpsc::Sender;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InventoryType {
    Error = 0,
    Transaction = 1,
    Block = 2,
    FilteredBlock = 3,
    CompactBlock = 4,
    WitnessTransaction = 0x40000001,
    WitnessBlock = 0x40000002,
    FilteredWitnessBlock = 0x40000003,
}

impl InventoryType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => InventoryType::Error,
            1 => InventoryType::Transaction,
            2 => InventoryType::Block,
            3 => InventoryType::FilteredBlock,
            4 => InventoryType::CompactBlock,
            0x40000001 => InventoryType::WitnessTransaction,
            0x40000002 => InventoryType::WitnessBlock,
            0x40000003 => InventoryType::FilteredWitnessBlock,
            _ => InventoryType::Error,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            InventoryType::Error => "ERROR",
            InventoryType::Transaction => "TX",
            InventoryType::Block => "BLOCK",
            InventoryType::FilteredBlock => "FILTERED_BLOCK",
            InventoryType::CompactBlock => "COMPACT_BLOCK",
            InventoryType::WitnessTransaction => "WITNESS_TX",
            InventoryType::WitnessBlock => "WITNESS_BLOCK",
            InventoryType::FilteredWitnessBlock => "FILTERED_WITNESS_BLOCK",
        }
    }
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub inv_type: InventoryType,
    pub hash: [u8; 32],
}

impl InventoryItem {
    pub fn hash_hex(&self) -> String {
        let mut reversed = self.hash;
        reversed.reverse();
        hex::encode(reversed)
    }
}

#[derive(Debug)]
pub struct BitcoinClient {
    stream: Option<TcpStream>,
    connected_addr: Option<SocketAddr>,
    handshake_complete: bool,
    version_received: bool,
    verack_received: bool,
    seen_inventory: std::collections::HashSet<[u8; 32]>,
    pub peer_db: PeerDatabase,
    pub log_tx: Option<Sender<LogMessage>>,
}

impl BitcoinClient {
    pub fn new_with_logger(log_tx: Sender<LogMessage>) -> Self {
        let peer_db = PeerDatabase::load_from_file("peers.json");
        BitcoinClient {
            stream: None,
            connected_addr: None,
            handshake_complete: false,
            version_received: false,
            verack_received: false,
            seen_inventory: std::collections::HashSet::new(),
            peer_db,
            log_tx: Some(log_tx),
        }
    }

    pub fn connect(&mut self) -> std::io::Result<()> {
        let addr_str = "seed.bitcoin.sipa.be:8333";
        if let Some(ref tx) = self.log_tx {
            log(tx, LogLevel::Info, Event::Custom(format!("Resolvendo {}", addr_str)));
        }
        let socket_addrs: Vec<_> = addr_str.to_socket_addrs()?.collect();
        if let Some(ref tx) = self.log_tx {
            log(tx, LogLevel::Debug, Event::Custom(format!("Endereços resolvidos: {:?}", socket_addrs)));
        }
        for addr in &socket_addrs {
            if let Some(ref tx) = self.log_tx {
                log(tx, LogLevel::Info, Event::Custom(format!("Tentando conectar em {}", addr)));
            }
            match TcpStream::connect_timeout(addr, Duration::from_secs(10)) {
                Ok(s) => {
                    if let Some(ref tx) = self.log_tx {
                        log(tx, LogLevel::Info, Event::Connected(*addr));
                    }
                    s.set_read_timeout(Some(Duration::from_secs(30)))?;
                    self.stream = Some(s);
                    self.connected_addr = Some(*addr);
                    break;
                }
                Err(e) => {
                    if let Some(ref tx) = self.log_tx {
                        log(tx, LogLevel::Warn, Event::FailedConnection(*addr, e.to_string()));
                    }
                    continue;
                }
            }
        }
        if self.stream.is_none() {
            if let Some(ref tx) = self.log_tx {
                log(tx, LogLevel::Error, Event::Custom("Não foi possível conectar a nenhum endereço".into()));
            }
            return Err(std::io::Error::new(ErrorKind::ConnectionRefused, "Could not connect to any address"));
        }
        if let Some(ref tx) = self.log_tx {
            log(tx, LogLevel::Info, Event::Custom(format!("Conexão estabelecida com {}", self.connected_addr.unwrap())));
        }
        Ok(())
    }
    
    pub fn start_handshake(&mut self) -> std::io::Result<()> {
        if let Some(ref tx) = self.log_tx {
            log(tx, LogLevel::Debug, Event::Custom("Enviando mensagem version".into()));
        }
        let version_payload = build_version_payload(self.connected_addr.unwrap());
        self.send_message("version", &version_payload)?;
        Ok(())
    }
    
    fn handle_message(&mut self, command: &str, payload: &[u8]) -> std::io::Result<()> {
        match command {
            "version" => {
                println!("   📋 Version message received");
                self.version_received = true;
    
                if payload.len() >= 4 {
                    let version = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
                    println!("   🔢 Protocol version: {}", version);
                }
    
                println!("📤 Sending verack...");
                self.send_message("verack", &[])?;
            }
            "verack" => {
                println!("   ✅ Verack received");
                self.verack_received = true;
            }
            "ping" => {
                println!("   🏓 Ping received");
                if payload.len() >= 8 {
                    println!("📤 Sending pong...");
                    self.send_message("pong", payload)?;
                }
            }
            "pong" => {
                println!("   🏓 Pong received");
            }
            "inv" => {
                println!("   📦 Inventory message received");
                self.handle_inv_message(payload)?;
            }
            "addr" => {
                println!("   📍 Address list received");
                let addresses = parse_addr_message(payload);
                println!("   📊 Successfully parsed {} addresses", addresses.len());
                // Registra cada peer na base de dados
                for addr in &addresses {
                    self.peer_db.register_peer(*addr, None);
                }
                // Opcional: salve imediatamente após receber novos peers
                self.peer_db.save_to_file("peers.json");
            }
            "alert" => {
                println!("   ⚠️  Alert message received (ignoring)");
            }
            "sendheaders" => {
                println!("   📋 SendHeaders message received");
            }
            "sendcmpct" => {
                println!("   📦 SendCmpct message received");
            }
            "feefilter" => {
                println!("   💰 FeeFilter message received");
            }
            _ => {
                println!("   ❓ Unknown command: {} ({} bytes)", command, payload.len());
            }
        }
    
        Ok(())
    }
    
    fn handle_inv_message(&mut self, payload: &[u8]) -> std::io::Result<()> {
        let inventory_items = parse_inv_message(payload);
        
        if inventory_items.is_empty() {
            println!("   ⚠️  Empty inventory message");
            return Ok(());
        }
        
        println!("   📊 Received {} inventory items:", inventory_items.len());
        
        let mut items_to_request = Vec::new();
        
        for (i, item) in inventory_items.iter().enumerate() {
            let hash_str = item.hash_hex();
            println!("     {}: {} - {}", i + 1, item.inv_type.name(), hash_str);
            
            // Check if we've already seen this item
            if !self.seen_inventory.contains(&item.hash) {
                self.seen_inventory.insert(item.hash);
                
                // Decide which types of inventory we want to request
                match item.inv_type {
                    InventoryType::Transaction | InventoryType::WitnessTransaction => {
                        // Request transactions (you might want to limit this)
                        if items_to_request.len() < 10 { // Limit to first 10 transactions
                            items_to_request.push(item.clone());
                        }
                    }
                    InventoryType::Block | InventoryType::WitnessBlock => {
                        // Request blocks (be careful, blocks are large!)
                        if items_to_request.len() < 3 { // Limit to first 3 blocks
                            items_to_request.push(item.clone());
                        }
                    }
                    InventoryType::CompactBlock => {
                        // Request compact blocks
                        items_to_request.push(item.clone());
                    }
                    _ => {
                        // Skip other types for now
                        println!("     → Skipping {} (not requesting)", item.inv_type.name());
                    }
                }
            } else {
                println!("     → Already seen this item, skipping");
            }
        }
        
        // Send getdata request for items we want
        if !items_to_request.is_empty() {
            println!("   📤 Requesting {} items via getdata...", items_to_request.len());
            let getdata_payload = build_getdata_payload(&items_to_request);
            self.send_message("getdata", &getdata_payload)?;
        } else {
            println!("   ✅ No new items to request");
        }
        
        Ok(())
    }
    
    fn is_connection_alive(&mut self) -> std::io::Result<bool> {
        if let Some(stream) = &mut self.stream {
            let mut peek_buf = [0u8; 1];
            stream.set_nonblocking(true)?;
            let result = match stream.peek(&mut peek_buf) {
                Ok(0) => false, // Connection closed
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => true, // No data but alive
                Err(_) => false, // Connection error
                Ok(_) => true,   // Data available
            };
            stream.set_nonblocking(false)?;
            Ok(result)
        } else {
            Ok(false)
        }
    }
    
    fn send_message(&mut self, command: &str, payload: &[u8]) -> std::io::Result<()> {
        if let Some(stream) = &mut self.stream {
            let header = MessageHeader::new(command, payload);
            let mut message = header.to_bytes();
            message.extend_from_slice(payload);
            
            stream.write_all(&message)?;
            stream.flush()?;
            
            println!("📤 Sent: {} ({} bytes payload)", command, payload.len());
        }
        
        Ok(())
    }
    
    fn read_message(&mut self) -> std::io::Result<Option<(MessageHeader, Vec<u8>)>> {
        if let Some(stream) = &mut self.stream {
            stream.set_nonblocking(true)?;
            let mut header_buf = [0u8; 24];
            let header_result = stream.read_exact(&mut header_buf);
            stream.set_nonblocking(false)?;
            
            match header_result {
                Ok(_) => {}
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
            
            let header = MessageHeader::from_bytes(&header_buf)
                .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "Invalid message header"))?;
            
            if header.magic != MAGIC {
                return Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid magic bytes"));
            }
            
            let mut payload = vec![0u8; header.payload_size as usize];
            if header.payload_size > 0 {
                stream.read_exact(&mut payload)?;
                
                let computed_checksum = sha256d(&payload);
                if header.checksum != computed_checksum[0..4] {
                    return Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid checksum"));
                }
            }
            
            Ok(Some((header, payload)))
        } else {
            Ok(None)
        }
    }
    
    pub fn soft_stop(&mut self) -> std::io::Result<()> {
        if let Some(stream) = &mut self.stream {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        self.stream = None;
        self.connected_addr = None;
        self.handshake_complete = false;
        self.version_received = false;
        self.verack_received = false;
        Ok(())
    }
   
    pub fn message_loop_with_channel(&mut self, tx: &Sender<String>) -> std::io::Result<()> {
        let mut getaddr_sent = false;
        let mut message_count = 0;
        let max_messages = 500000;

        loop {
            message_count += 1;
            if message_count > max_messages {
                let _ = tx.send(format!("\n🎯 Processed {} messages, ending demo", max_messages));
                break;
            }

            if !self.is_connection_alive()? {
                let _ = tx.send("🔌 Connection closed by peer".to_string());
                break;
            }

            match self.read_message() {
                Ok(Some((header, payload))) => {
                    let command = header.command_str();
                    let _ = tx.send(format!("⬇️ Received: {} ({} bytes)", command, payload.len()));

                    // Opcional: envie mais detalhes se quiser
                    self.handle_message(&command, &payload)?;

                    if self.version_received && self.verack_received && !self.handshake_complete {
                        let _ = tx.send("\n🤝 Handshake complete!".to_string());
                        self.handshake_complete = true;
                    }
                    if self.handshake_complete && !getaddr_sent {
                        let _ = tx.send("📤 Requesting peer addresses...".to_string());
                        self.send_message("getaddr", &[])?;
                        getaddr_sent = true;
                    }
                }
                Ok(_none) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    let _ = tx.send(format!("❌ Error reading message: {}", e));
                    break;
                }
            }
            if self.handshake_complete && getaddr_sent {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
        Ok(())
    }

    pub async fn _connect_async(&mut self) -> std::io::Result<()> {
    use tokio::net::TcpStream as TokioTcpStream; 

    let addr_str = "seed.bitcoin.sipa.be:8333";
    if let Some(ref tx) = self.log_tx {
        log(tx, LogLevel::Info, Event::Custom(format!("Resolvendo {}", addr_str)));
    }
    let socket_addrs: Vec<_> = tokio::net::lookup_host(addr_str).await?.collect();
    if let Some(ref tx) = self.log_tx {
        log(tx, LogLevel::Debug, Event::Custom(format!("Endereços resolvidos: {:?}", socket_addrs)));
    }
    for addr in &socket_addrs {
        if let Some(ref tx) = self.log_tx {
            log(tx, LogLevel::Info, Event::Custom(format!("Tentando conectar em {}", addr)));
        }
        match TokioTcpStream::connect(addr).await {
            Ok(_s) => {
                if let Some(ref tx) = self.log_tx {
                    log(tx, LogLevel::Info, Event::Connected(*addr));
                }
                self.connected_addr = Some(*addr);
                break;
            }
            Err(e) => {
                if let Some(ref tx) = self.log_tx {
                    log(tx, LogLevel::Warn, Event::FailedConnection(*addr, e.to_string()));
                }
                continue;
            }
        }
    }
    if self.connected_addr.is_none() {
        if let Some(ref tx) = self.log_tx {
            log(tx, LogLevel::Error, Event::Custom("Não foi possível conectar a nenhum endereço".into()));
        }
        return Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Could not connect to any address"));
    }
    if let Some(ref tx) = self.log_tx {
        log(tx, LogLevel::Info, Event::Custom(format!("Conexão estabelecida com {}", self.connected_addr.unwrap())));
    }
    Ok(())
}
    
}

impl Clone for BitcoinClient {
    fn clone(&self) -> Self {
        BitcoinClient {
            stream: self.stream.as_ref().map(|s| s.try_clone().expect("Falha ao clonar stream")),
            connected_addr: self.connected_addr,
            handshake_complete: self.handshake_complete,
            version_received: self.version_received,
            verack_received: self.verack_received,
            seen_inventory: self.seen_inventory.clone(),
            peer_db: self.peer_db.clone(),
            log_tx: self.log_tx.clone(),
        }
    }
}