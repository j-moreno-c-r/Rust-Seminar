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

    pub fn name(&self) -> &'static str {
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
