pub(crate) use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::BODY_BYTES, worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementRedo, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::REWRITE_DOMAIN, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::inspect_retirement_redo};

/// Intent for source root 4, segment 1, generation 2, 16 bytes, written
/// out field by field rather than through any encoder.
pub(crate) fn golden_intent() -> Vec<u8> {
    let mut bytes = vec![28, 0, 0, 0, 0, 0, 0, 0];
    bytes.extend_from_slice(b"store.physical.retirement.v2");
    bytes.push(1);
    bytes.extend_from_slice(&[4, 0, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&[2, 0, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&[16, 0, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&[0; 88]);
    bytes
}
