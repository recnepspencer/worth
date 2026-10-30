#[allow(unused_imports)]
use crate::physical_work_obligation_validation::envelope::*;

#[test]
fn frozen_literals_seal_family_specific_views_and_validation_records() {
    let store = store(STORE_BYTES);
    let cases = [
        (operation_3(), 3, OPERATION_3_SHA),
        (operation_4(), 4, OPERATION_4_SHA),
    ];
    for (bytes, operation, covered_digest) in cases {
        assert_eq!(bytes.len(), PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES);
        assert_eq!(bytes[8], PHYSICAL_WORK_OBLIGATION_V6_VERSION);
        assert_eq!(independent_sha256(&bytes[..128]), covered_digest);
        assert_eq!(&bytes[128..160], covered_digest.as_slice());

        let scope = scope(store, identity(1, 2, operation));
        let artifact =
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
        let other_incarnation = bytes;
        let (validation, counters) = validate_physical_work_obligation(artifact, scope);
        let worth_store_physical_integrity::PhysicalWorkObligationIntegrityValidation::Intact(
            validated,
        ) = validation
        else {
            panic!("frozen physical-work literal was rejected")
        };
        assert_eq!(validated.scope(), scope);
        assert_eq!(validated.identity(), identity(1, 2, operation));
        assert!(validated.matches_input(artifact));
        assert!(!validated.matches_input(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                &other_incarnation,
            )
        ));
        match operation {
            3 => {
                assert_eq!(
                    validated.operation_code(),
                    worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::DurabilityBarrier
                );
                assert_eq!(
                    validated.target(),
                    worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode::RecordNamespaceSynchronization
                );
                assert_eq!(validated.payload_digest(), None);
            }
            4 => {
                assert_eq!(
                    validated.operation_code(),
                    worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode::WalAppend
                );
                assert_eq!(
                    validated.target(),
                    worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode::WalArtifactInterval {
                        segment: 7,
                        generation: 8,
                        offset: 9,
                        byte_count: 10,
                    }
                );
                assert_eq!(validated.payload_digest(), Some([0xab; 32]));
            }
            _ => unreachable!(),
        }
        let record = validated.into_validation_record();
        assert!(record.matches_scope(scope));
        assert_eq!(
            record.mechanism(),
            worth_store_physical_integrity::PhysicalIntegrityValidationMechanism::Sha256V1
        );
        assert_eq!(
            record.byte_range_digest(),
            worth_store_physical_integrity::PhysicalIntegrityValidationDigest::sha256(
                independent_sha256(&bytes)
            )
        );
        assert_eq!(
            counters.family(),
            worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::PhysicalWorkObligation
        );
        assert_eq!(counters.inspected_frames(), 1);
        assert_eq!(counters.inspected_bytes(), 160);
        assert_eq!(counters.intact_frames(), 1);
        assert_eq!(counters.rejected_frames(), 0);
    }
}
