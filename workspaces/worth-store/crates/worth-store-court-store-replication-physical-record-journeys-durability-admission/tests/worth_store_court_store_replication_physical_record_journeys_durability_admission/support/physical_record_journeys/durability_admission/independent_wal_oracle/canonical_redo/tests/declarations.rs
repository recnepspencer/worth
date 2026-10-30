pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::inspect_recovery_projection, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::write_field, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::canonical_redo::CURRENT_PROJECTION_DOMAIN};

pub(crate) fn projection(domain: &[u8], source_copy: bool, operation: Option<&[u8]>) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_field(&mut bytes, domain);
    bytes.extend_from_slice(&4_u64.to_le_bytes());
    write_field(&mut bytes, b"root");
    bytes.extend_from_slice(&1_u64.to_le_bytes());
    write_field(&mut bytes, &[7; 24]);
    if source_copy {
        bytes.push(1);
        write_field(&mut bytes, b"copy-intent");
        bytes.extend_from_slice(&3_u64.to_le_bytes());
        bytes.extend_from_slice(&[9; 32]);
    } else {
        bytes.push(0);
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        write_field(&mut bytes, b"frame");
    }
    bytes.extend_from_slice(&1_u64.to_le_bytes());
    write_field(&mut bytes, b"placement");
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    bytes.extend_from_slice(&0_u64.to_le_bytes());
    if let Some(operation) = operation {
        write_field(&mut bytes, operation);
    }
    bytes
}
