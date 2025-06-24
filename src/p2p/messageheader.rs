use crate::p2p::utils::{sha256d, MAGIC};
#[derive(Debug)]
pub struct MessageHeader {
    pub magic: [u8; 4],
    pub command: [u8; 12],
    pub payload_size: u32,
    pub checksum: [u8; 4],
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
