mod support;

use support::independent_sha256;
use worth_store_physical_format::physical_work_obligation::{
    decode_physical_work_obligation_v6, encode_physical_work_obligation_v6,
    PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
    PhysicalWorkObligationV6,
};

const OPERATION_3_HEX: &str = "575045464645435406060000000000000102030405060708090a0b0c0d0e0f10010000000000000002000000000000000300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000b2a18aee7dbc138ec738507b8fcd7f722280d499d9bf0f3962294fccfe50bd0c";
const OPERATION_4_HEX: &str = "575045464645435406050000000000000102030405060708090a0b0c0d0e0f1001000000000000000200000000000000040000000000000009000000000000000a00000000000000abababababababababababababababababababababababababababababababab0601000000000000070000000000000008000000000000004cad4be6809b3b053cf2951815f20c0d1cb2649b87a61b582c1a0a3a89010b90";
const OPERATION_3_SHA: &str = "b2a18aee7dbc138ec738507b8fcd7f722280d499d9bf0f3962294fccfe50bd0c";
const OPERATION_4_SHA: &str = "4cad4be6809b3b053cf2951815f20c0d1cb2649b87a61b582c1a0a3a89010b90";

#[test]
fn physical_work_v6_matches_both_frozen_literals_and_filenames() {
    let store = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let cases = [
        (
            PhysicalWorkObligationV6::new(
                store,
                1,
                2,
                3,
                PhysicalWorkObligationOperationCode::DurabilityBarrier,
                PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization,
                None,
            )
            .unwrap(),
            OPERATION_3_HEX,
            OPERATION_3_SHA,
            "effect-0000000000000001-0000000000000002-0000000000000003.pending",
        ),
        (
            PhysicalWorkObligationV6::new(
                store,
                1,
                2,
                4,
                PhysicalWorkObligationOperationCode::WalAppend,
                PhysicalWorkObligationTargetCode::WalArtifactInterval {
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
        (PhysicalWorkObligationOperationCode::ArtifactRangeRead, 1),
        (PhysicalWorkObligationOperationCode::ArtifactRangeWrite, 2),
        (PhysicalWorkObligationOperationCode::ArtifactPublication, 3),
        (PhysicalWorkObligationOperationCode::ArtifactMetadataRead, 4),
        (PhysicalWorkObligationOperationCode::WalAppend, 5),
        (PhysicalWorkObligationOperationCode::DurabilityBarrier, 6),
        (PhysicalWorkObligationOperationCode::CheckpointCapture, 7),
        (PhysicalWorkObligationOperationCode::WalReclamation, 8),
        (PhysicalWorkObligationOperationCode::RootPublication, 9),
    ];
    for (operation_code, expected_tag) in cases {
        let value = PhysicalWorkObligationV6::new(
            [1; 16],
            1,
            1,
            u64::from(expected_tag),
            operation_code,
            PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization,
            None,
        )
        .unwrap();
        let encoded = encode_physical_work_obligation_v6(value);
        assert_eq!(encoded[9], expected_tag);
        assert_eq!(decode_physical_work_obligation_v6(&encoded), Ok(value));
    }
}

fn literal(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).expect("literal hex byte")
        })
        .collect()
}
