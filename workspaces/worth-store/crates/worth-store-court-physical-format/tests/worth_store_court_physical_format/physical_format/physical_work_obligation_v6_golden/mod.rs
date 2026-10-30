#[allow(unused_imports)]
use crate::physical_work_obligation_v6_golden::*;

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
