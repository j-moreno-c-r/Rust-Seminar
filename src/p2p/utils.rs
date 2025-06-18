
use sha2::{Digest, Sha256};
use crate::p2p::p2p_client::{InventoryItem, InventoryType};
use std::net::{SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};


pub const MAGIC: [u8; 4] = [0xF9, 0xBE, 0xB4, 0xD9];

pub fn sha256d(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(&first);
    second.into()
}

pub fn parse_inv_message(payload: &[u8]) -> Vec<InventoryItem> {
    let mut items = Vec::new();
    
    if payload.is_empty() {
        return items;
    }
    
    // Parse compact size integer for count
    let (count, mut offset) = parse_compact_size(payload);
    
    if count == 0 {
        return items;
    }
    
    // Each inventory item is 36 bytes (4 bytes type + 32 bytes hash)
    for _ in 0..count {
        if offset + 36 > payload.len() {
            break;
        }
        
        // Read type (4 bytes, little endian)
        let type_bytes = &payload[offset..offset + 4];
        let inv_type_raw = u32::from_le_bytes([type_bytes[0], type_bytes[1], type_bytes[2], type_bytes[3]]);
        let inv_type = InventoryType::from_u32(inv_type_raw);
        offset += 4;
        
        // Read hash (32 bytes)
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&payload[offset..offset + 32]);
        offset += 32;
        
        items.push(InventoryItem {
            inv_type,
            hash,
        });
    }
    
    items
}

pub fn build_getdata_payload(items: &[InventoryItem]) -> Vec<u8> {
    let mut payload = Vec::new();
    
    // Add compact size for count
    let count = items.len();
    if count < 0xFD {
        payload.push(count as u8);
    } else if count <= 0xFFFF {
        payload.push(0xFD);
        payload.extend((count as u16).to_le_bytes());
    } else if count <= 0xFFFFFFFF {
        payload.push(0xFE);
        payload.extend((count as u32).to_le_bytes());
    } else {
        payload.push(0xFF);
        payload.extend((count as u64).to_le_bytes());
    }
    
    // Add each inventory item
    for item in items {
        // Add type (4 bytes, little endian)
        payload.extend((item.inv_type as u32).to_le_bytes());
        
        // Add hash (32 bytes)
        payload.extend(item.hash);
    }
    
    payload
}

pub fn build_version_payload(peer_addr: SocketAddr) -> Vec<u8> {
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

pub fn parse_addr_message(payload: &[u8]) -> Vec<SocketAddr> {
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

pub fn parse_compact_size(data: &[u8]) -> (usize, usize) {
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