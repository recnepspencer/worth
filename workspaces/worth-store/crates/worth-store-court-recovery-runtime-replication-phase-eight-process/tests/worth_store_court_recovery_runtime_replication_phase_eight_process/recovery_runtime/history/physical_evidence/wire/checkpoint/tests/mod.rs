#[allow(unused_imports)]
use crate::phase_eight_process::history::parent_oracle::wire::checkpoint::tests::*;
use {sha2::Digest, sha2::Sha256};

use {
    crate::phase_eight_process::history::parent_oracle::wire::checkpoint::crc32c,
    crate::phase_eight_process::history::parent_oracle::wire::checkpoint::observe_checkpoint,
    crate::phase_eight_process::history::parent_oracle::wire::checkpoint::validated_binding_payloads,
    crate::phase_eight_process::history::parent_oracle::wire::checkpoint::CHECKPOINT_PREFIX_BYTES,
};

#[test]
fn certified_envelope_and_legacy_envelopes_remain_observable() {
    for schema in [1, 2, 3] {
        let certificates = if schema == 3 { &[6, 7][..] } else { &[][..] };
        let bytes = stream(schema, certificates);
        let facts = observe_checkpoint(&bytes).expect("lawful checkpoint envelope");
        assert_eq!(facts.sequence, 1);
        assert_eq!(facts.covered, (11, 29));
        assert_eq!(facts.durable, 29);
    }
}

#[test]
fn certified_certificate_crc_and_aggregate_are_independent_checks() {
    let mut bytes = stream(3, &[6, 7]);
    let first_certificate = 164 + 36;
    bytes[first_certificate + CHECKPOINT_PREFIX_BYTES] ^= 1;
    assert!(observe_checkpoint(&bytes).is_none(), "certificate CRC");

    let payload_bytes = u32::from_le_bytes(
        bytes[first_certificate + 12..first_certificate + 16]
            .try_into()
            .unwrap(),
    ) as usize;
    let checksum = crc32c(
        &bytes[first_certificate..first_certificate + CHECKPOINT_PREFIX_BYTES + payload_bytes],
    );
    bytes[first_certificate + CHECKPOINT_PREFIX_BYTES + payload_bytes
        ..first_certificate + CHECKPOINT_PREFIX_BYTES + payload_bytes + 4]
        .copy_from_slice(&checksum.to_le_bytes());
    assert!(
        observe_checkpoint(&bytes).is_none(),
        "valid record CRC cannot override the footer certificate aggregate"
    );
}

#[test]
fn mixed_record_schema_and_misordered_tier_are_not_selected() {
    let mut mixed = stream(3, &[6, 7]);
    let compaction = 164;
    mixed[compaction + 8] = 1;
    let checksum = crc32c(&mixed[compaction..compaction + 16 + 16]);
    mixed[compaction + 32..compaction + 36].copy_from_slice(&checksum.to_le_bytes());
    assert!(observe_checkpoint(&mixed).is_none());

    assert!(observe_checkpoint(&stream(3, &[7, 6])).is_none());
    assert!(observe_checkpoint(&stream(3, &[6, 6])).is_none());
}

#[test]
fn certified_envelope_exposes_only_aggregate_bound_binding_payloads() {
    let binding = b"independent-kind-four-binding";
    let bytes = stream_with_binding(3, &[6, 7], Some(binding));
    assert!(observe_checkpoint(&bytes).is_some());
    assert_eq!(validated_binding_payloads(&bytes), Some(vec![&binding[..]]));
}
