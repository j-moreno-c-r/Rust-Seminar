use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::io::{Write, Read, ErrorKind};
use sha2::{Digest, Sha256};

const MAGIC: [u8; 4] = [0xF9, 0xBE, 0xB4, 0xD9];

#[derive(Debug)]
pub struct MessageHeader {
    magic: [u8; 4],
    command: [u8; 12],
    payload_size: u32,
    checksum: [u8; 4],
}

impl MessageHeader {
    pub fn new(command: &str, payload: &[u8]) -> Self {
        let mut cmd_bytes = [0u8; 12];
        let cmd_str = command.as_bytes();
        cmd_bytes[..cmd_str.len()].copy_from_slice(cmd_str);
        
        let checksum = sha256d(payload);
        let mut checksum_bytes = [0u8; 4];
        checksum_bytes.copy_from_slice(&checksum[0..4]);
        
        MessageHeader {
            magic: MAGIC,
            command: cmd_bytes,
            payload_size: payload.len() as u32,
            checksum: checksum_bytes,
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.magic);
        bytes.extend(&self.command);
        bytes.extend(self.payload_size.to_le_bytes());
        bytes.extend(&self.checksum);
        bytes
    }
    
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 24 {
            return None;
        }
        
        let mut magic = [0u8; 4];
        let mut command = [0u8; 12];
        let mut checksum = [0u8; 4];
        
        magic.copy_from_slice(&data[0..4]);
        command.copy_from_slice(&data[4..16]);
        let payload_size = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        checksum.copy_from_slice(&data[20..24]);
        
        Some(MessageHeader {
            magic,
            command,
            payload_size,
            checksum,
        })
    }
    
    pub fn command_str(&self) -> String {
        let end = self.command.iter().position(|&b| b == 0).unwrap_or(12);
        String::from_utf8_lossy(&self.command[..end]).to_string()
    }
}

pub struct BitcoinClient {
    stream: Option<TcpStream>,
    connected_addr: Option<SocketAddr>,
    handshake_complete: bool,
    version_received: bool,
    verack_received: bool,
}

impl BitcoinClient {
    pub fn new() -> Self {
        BitcoinClient {
            stream: None,
            connected_addr: None,
            handshake_complete: false,
            version_received: false,
            verack_received: false,
        }
    }
    
    pub fn run(&mut self) -> std::io::Result<()> {
        println!("🚀 Starting Bitcoin P2P Client");
        
        self.connect()?;
        self.start_handshake()?;
        self.message_loop()?;
        
        Ok(())
    }
    
    fn connect(&mut self) -> std::io::Result<()> {
        // Step 1: Resolve DNS and explore addresses
        println!("\n🔍 Resolving seed.bitcoin.sipa.be:8333...");
        let socket_addrs: Vec<_> = "seed.bitcoin.sipa.be:8333"
            .to_socket_addrs()?
            .collect();
        
        println!("📍 Resolved addresses:");
        for (i, addr) in socket_addrs.iter().enumerate() {
            println!("  {}: {} ({})", i + 1, addr, 
                     if addr.is_ipv4() { "IPv4" } else { "IPv6" });
        }
        
        // Try to connect to each address until one succeeds
        for addr in &socket_addrs {
            println!("\n🔌 Attempting to connect to {}...", addr);
            match TcpStream::connect_timeout(addr, Duration::from_secs(10)) {
                Ok(s) => {
                    println!("✅ Connected successfully!");
                    // Set a reasonable read timeout to prevent hanging
                    s.set_read_timeout(Some(Duration::from_secs(30)))?;
                    self.stream = Some(s);
                    self.connected_addr = Some(*addr);
                    break;
                }
                Err(e) => {
                    println!("❌ Failed to connect: {}", e);
                    continue;
                }
            }
        }
        
        if self.stream.is_none() {
            return Err(std::io::Error::new(ErrorKind::ConnectionRefused, "Could not connect to any address"));
        }
        
        println!("🎯 Using connection to: {}", self.connected_addr.unwrap());
        Ok(())
    }
    
    fn start_handshake(&mut self) -> std::io::Result<()> {
        // Send version message
        println!("\n📤 Sending version message...");
        let version_payload = build_version_payload(self.connected_addr.unwrap());
        self.send_message("version", &version_payload)?;
        Ok(())
    }
    
    fn message_loop(&mut self) -> std::io::Result<()> {
        let mut getaddr_sent = false;
        let mut message_count = 0;
        let max_messages = 500000; 
        
        loop {
            message_count += 1;
            if message_count > max_messages {
                println!("\n🎯 Processed {} messages, ending demo", max_messages);
                break;
            }
            
            if !self.is_connection_alive()? {
                println!("🔌 Connection closed by peer");
                break;
            }
            
            match self.read_message() {
                Ok(Some((header, payload))) => {
                    let command = header.command_str();
                    println!("📥 Received: {} ({} bytes)", command, payload.len());
                    
                    self.handle_message(&command, &payload)?;
                    
                    // Send getaddr after handshake completes, but only once
                    if self.version_received && self.verack_received && !self.handshake_complete {
                        println!("\n🤝 Handshake complete!");
                        self.handshake_complete = true;
                    }
                    
                    if self.handshake_complete && !getaddr_sent {
                        println!("📤 Requesting peer addresses...");
                        self.send_message("getaddr", &[])?;
                        getaddr_sent = true;
                    }
                }
                Ok(None) => {
                    // No message available, small delay
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    println!("❌ Error reading message: {}", e);
                    break;
                }
            }
            
            // If we've completed handshake and sent getaddr, wait a bit more for responses
            if self.handshake_complete && getaddr_sent {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
        
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
            "addr" => {
                println!("   📍 Address list received");
                let addresses = parse_addr_message(payload);
                println!("   📊 Successfully parsed {} addresses", addresses.len());
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
            // Try to read header first - use non-blocking for availability check
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
                
                // Verify checksum
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
}

fn build_version_payload(peer_addr: SocketAddr) -> Vec<u8> {
    let mut payload = Vec::new();
    
    // Protocol version
    payload.extend(70015u32.to_le_bytes());
    
    // Services (NODE_NETWORK)
    payload.extend(1u64.to_le_bytes());
    
    // Timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    payload.extend(timestamp.to_le_bytes());
    
    // Peer address (recipient)
    payload.extend(1u64.to_le_bytes()); // Services
    match peer_addr {
        SocketAddr::V4(v4) => {
            payload.extend([0x00; 12]);          // IPv6 prefix
            payload.extend([0xFF, 0xFF]);        // IPv4 marker
            payload.extend(v4.ip().octets());    // IP address
            payload.extend(v4.port().to_be_bytes()); // Port
        }
        SocketAddr::V6(v6) => {
            payload.extend(v6.ip().octets());    // IPv6 address
            payload.extend(v6.port().to_be_bytes()); // Port
        }
    }
    
    // My address (sender)
    payload.extend(0u64.to_le_bytes());  // Services
    payload.extend([0x00; 16]);          // IPv6 (all zeros for local)
    payload.extend(0u16.to_be_bytes());  // Port
    
    // Nonce
    payload.extend(123456789u64.to_le_bytes());
    
    // User agent (empty)
    payload.push(0x00); // Compact size (length 0)
    
    // Start height
    payload.extend(0i32.to_le_bytes());
    
    // Relay flag
    payload.push(0x01); // True
    
    payload
}

fn parse_addr_message(payload: &[u8]) -> Vec<SocketAddr> {
    let mut addresses = Vec::new();
    
    if payload.is_empty() {
        println!("   ⚠️  Empty address payload");
        return addresses;
    }
    
    // Parse compact size integer for count
    let (count, mut offset) = parse_compact_size(payload);
    println!("   📊 Address count: {}", count);
    
    if count == 0 {
        return addresses;
    }
    
    let addresses_to_show = count.min(10); // Show first 10
    
    for i in 0..addresses_to_show {
        if offset + 30 > payload.len() {
            println!("   ⚠️  Insufficient data for address {}", i + 1);
            break;
        }
        
        // Skip timestamp (4 bytes)
        offset += 4;
        
        // Skip services (8 bytes)  
        offset += 8;
        
        // Read IP address (16 bytes)
        let ip_bytes = &payload[offset..offset + 16];
        offset += 16;
        
        // Read port (2 bytes, big endian)
        if offset + 2 > payload.len() {
            break;
        }
        let port = u16::from_be_bytes([payload[offset], payload[offset + 1]]);
        offset += 2;
        
        // Parse IP address
        if ip_bytes[0..12] == [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF] {
            // IPv4 address
            let ipv4 = std::net::Ipv4Addr::new(
                ip_bytes[12], ip_bytes[13], ip_bytes[14], ip_bytes[15]
            );
            let addr = SocketAddr::V4(std::net::SocketAddrV4::new(ipv4, port));
            println!("     {}: {} (IPv4)", i + 1, addr);
            addresses.push(addr);
        } else {
            // IPv6 address
            let ipv6 = std::net::Ipv6Addr::from(
                *<&[u8; 16]>::try_from(ip_bytes).unwrap()
            );
            let addr = SocketAddr::V6(std::net::SocketAddrV6::new(ipv6, port, 0, 0));
            println!("     {}: {} (IPv6)", i + 1, addr);
            addresses.push(addr);
        }
    }
    
    if count > addresses_to_show {
        println!("     ... and {} more addresses", count - addresses_to_show);
    }
    
    addresses
}

fn parse_compact_size(data: &[u8]) -> (usize, usize) {
    if data.is_empty() {
        return (0, 0);
    }
    
    let first_byte = data[0];
    match first_byte {
        0x00..=0xFC => (first_byte as usize, 1),
        0xFD => {
            if data.len() >= 3 {
                let value = u16::from_le_bytes([data[1], data[2]]) as usize;
                (value, 3)
            } else {
                (0, 1)
            }
        }
        0xFE => {
            if data.len() >= 5 {
                let value = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
                (value, 5)
            } else {
                (0, 1)
            }
        }
        0xFF => {
            if data.len() >= 9 {
                let value = u64::from_le_bytes([
                    data[1], data[2], data[3], data[4],
                    data[5], data[6], data[7], data[8]
                ]) as usize;
                (value, 9)
            } else {
                (0, 1)
            }
        }
    }
}

fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    second.into()
}