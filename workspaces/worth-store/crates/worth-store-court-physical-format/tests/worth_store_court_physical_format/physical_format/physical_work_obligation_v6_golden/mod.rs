

#[allow(unused_imports)] use crate::physical_work_obligation_v6_golden::*;
use {crate::store_physical_format_support::independent_sha256};
use {worth_store_physical_format::physical_work_obligation::decode_physical_work_obligation_v6, worth_store_physical_format::physical_work_obligation::encode_physical_work_obligation_v6, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6};

#[test]
fn physical_work_v6_matches_both_frozen_literals_and_filenames() {
    let store = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let cases = [
        (
            worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6::new(
                store,
                1,
                2,
                3,
                worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::DurabilityBarrier,
                worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization,
                None,
            )
            .unwrap(),
            OPERATION_3_HEX,
            OPERATION_3_SHA,
            "effect-0000000000000001-0000000000000002-0000000000000003.pending",
        ),
        (
            worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6::new(
                store,
                1,
                2,
                4,
                worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::WalAppend,
                worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode::WalArtifactInterval {
                    segment: 7,
                    generation: 8,
                    offset: 9,
                    byte_count: 10,
                },
                Some([0xab; 32]),
            )
            .unwrap(),
            OPERATION_4_HEX,
            OPERATION_4_SHA,
            "effect-0000000000000001-0000000000000002-0000000000000004.pending",
        ),
    ];

    for (value, encoded_hex, checksum_hex, expected_name) in cases {
        let expected = literal(encoded_hex);
        let expected_checksum = literal(checksum_hex);
        assert_eq!(
            independent_sha256(&expected[..128]).as_slice(),
            expected_checksum
        );
        assert_eq!(
            encode_physical_work_obligation_v6(value),
            expected.as_slice()
        );
        assert_eq!(decode_physical_work_obligation_v6(&expected), Ok(value));
        assert_eq!(
            format!(
                "effect-{:016x}-{:016x}-{:016x}.pending",
                value.runtime(),
                value.generation(),
                value.operation()
            ),
            expected_name
        );
    }
}

#[test]
fn physical_work_v6_preserves_every_frozen_operation_tag() {
    let cases = [
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::ArtifactRangeRead, 1),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::ArtifactRangeWrite, 2),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::ArtifactPublication, 3),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::ArtifactMetadataRead, 4),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::WalAppend, 5),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::DurabilityBarrier, 6),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::CheckpointCapture, 7),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::WalReclamation, 8),
        (worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::RootPublication, 9),
    ];
    for (operation_code, expected_tag) in cases {
        let value = worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6::new(
            [1; 16],
            1,
            1,
            u64::from(expected_tag),
            operation_code,
            worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization,
            None,
        )
        .unwrap();
        let encoded = encode_physical_work_obligation_v6(value);
        assert_eq!(encoded[9], expected_tag);
        assert_eq!(decode_physical_work_obligation_v6(&encoded), Ok(value));
    }
}
