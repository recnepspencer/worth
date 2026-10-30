use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::BODY_BYTES, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::CANONICAL_DOMAIN, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::REWRITE_DOMAIN, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::inspect_rewrite_redo, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::sample_rewrite};

#[test]
fn rewrite_golden_vectors_reject_unknown_and_malformed_payloads() {
    let rewrite = inspect_rewrite_redo(&sample_rewrite()).unwrap();
    assert_eq!(rewrite.candidate_bytes, 4096);
    assert_eq!(rewrite.source_length, rewrite.destination_length);

    let mut truncated = sample_rewrite();
    truncated.pop();
    assert_eq!(
            inspect_rewrite_redo(&truncated),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::Truncated)
        );

    let mut unknown = (b"store.physical.unknown.v9".len() as u64)
        .to_le_bytes()
        .to_vec();
    unknown.extend_from_slice(b"store.physical.unknown.v9");
    unknown.extend_from_slice(&[0; BODY_BYTES]);
    assert_eq!(
            inspect_rewrite_redo(&unknown),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch)
        );

    let mut canonical = (CANONICAL_DOMAIN.len() as u64).to_le_bytes().to_vec();
    canonical.extend_from_slice(CANONICAL_DOMAIN);
    assert_eq!(
            inspect_rewrite_redo(&canonical),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::DomainMismatch)
        );

    let mut mismatch = sample_rewrite();
    let dest_len_at = 8 + REWRITE_DOMAIN.len() + 140;
    mismatch[dest_len_at..dest_len_at + 4].copy_from_slice(&4095_u32.to_le_bytes());
    assert_eq!(
            inspect_rewrite_redo(&mismatch),
            Err(worth_store_testkit_store::durability_admission::independent_wal_oracle::BindingInspectionDenial::InvalidFrame)
        );
}
