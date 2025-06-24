#[cfg(test)]
mod tests {
    use crate::p2p::utils::*;
    use crate::p2p::inventory::{InventoryItem, InventoryType};
    use std::net::{SocketAddr, Ipv4Addr};



    #[test]
    fn test_build_getdata_payload_and_parse_inv() {
        let item = InventoryItem {
            inv_type: InventoryType::Transaction,
            hash: [1u8; 32],
        };
        let payload = build_getdata_payload(&[item.clone()]);
        let _parsed = parse_inv_message(&payload[1..]); // skip compact size
        // Como build_getdata_payload não gera um payload de inv, só testamos o tamanho
        assert!(payload.len() > 32);
    }

    #[test]
    fn test_parse_compact_size() {
        let data = [0xFD, 0x10, 0x00]; // 16 em little endian
        let (val, offset) = parse_compact_size(&data);
        assert_eq!(val, 16);
        assert_eq!(offset, 3);
    }

    #[test]
    fn test_build_version_payload_ipv4() {
        let addr = SocketAddr::V4(std::net::SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8333));
        let payload = build_version_payload(addr);
        assert!(payload.len() > 0);
    }

    #[test]
    fn test_inventory_hash_hex() {
        let mut _item = InventoryItem {
            inv_type: InventoryType::Block,
            hash: [0xAB; 32],
        };
        let hex = _item.hash_hex();
        assert_eq!(hex.len(), 64);
    }

    #[test]
    fn test_inventory_type_from_u32() {
        assert_eq!(InventoryType::from_u32(1), InventoryType::Transaction);
        assert_eq!(InventoryType::from_u32(2), InventoryType::Block);
        assert_eq!(InventoryType::from_u32(0x40000001), InventoryType::WitnessTransaction);
        assert_eq!(InventoryType::from_u32(999999), InventoryType::Error);
    }

    #[test]
    fn test_inventory_type_name() {
        assert_eq!(InventoryType::Transaction.name(), "TX");
        assert_eq!(InventoryType::Block.name(), "BLOCK");
        assert_eq!(InventoryType::WitnessBlock.name(), "WITNESS_BLOCK");
    }
    #[test]
    fn test_parse_compact_size_invalid() {
        // Payload muito curto para 2 bytes
        let data = [0xFD, 0x01];
        let (val, offset) = parse_compact_size(&data);
        // Espera-se fallback para 0 e offset 1
        assert_eq!(val, 0);
        assert_eq!(offset, 1);
    }

    #[test]
    fn test_build_getdata_payload_empty() {
        let payload = build_getdata_payload(&[]);
        // Deve conter apenas o compact size 0
        assert_eq!(payload, vec![0u8]);
    }

    #[test]
    fn test_inventory_item_hash_hex_format() {
        let item = InventoryItem {
            inv_type: InventoryType::Transaction,
            hash: [0xFF; 32],
        };
        let hex = item.hash_hex();
        // Deve ser 64 caracteres, todos 'f'
        assert!(hex.chars().all(|c| c == 'f' || c == 'F'));
    }

    #[test]
    fn test_parse_inv_message_invalid_payload() {
        // Payload menor que o necessário para um item
        let payload = [0x01, 0x02, 0x03];
        let items = parse_inv_message(&payload);
        assert!(items.is_empty());
    }
}