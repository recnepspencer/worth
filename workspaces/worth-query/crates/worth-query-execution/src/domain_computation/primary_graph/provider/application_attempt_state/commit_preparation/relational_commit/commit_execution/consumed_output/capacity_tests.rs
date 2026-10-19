//! Exact precommit capacity evidence survives the existing provider mapping.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::provider_compare_denial::provider_session_denied,
    VerificationCapacityStop as Capacity, WorthQueryApplicationCommitDenialKind as Kind,
    WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::{
    WorthQueryProviderSessionDenialKind as ProviderKind, WorthQueryProviderSessionRecoveryPosture,
};
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;

#[test]
fn precommit_capacity_keeps_original_cause_and_pre_effect_posture() {
    for capacity in [
        Capacity::Meter(Stop::PreparationMemoryExhausted {
            required: 65,
            maximum: 64,
        }),
        Capacity::Meter(Stop::PreparationMemoryCounterOverflow),
        Capacity::Meter(Stop::RetainedCompanionCapacityExhausted {
            requested: 3,
            retained: 62,
            maximum: 64,
        }),
        Capacity::ScratchSizeOverflow,
        Capacity::ScratchAllocationRefused { bytes: 65 },
    ] {
        let WorthQueryProviderSessionCommitStop::PreEffectDenied(failure) =
            capacity_denied(capacity)
        else {
            panic!("verification capacity must refuse before effects");
        };
        assert_eq!(failure.kind(), ProviderKind::ProviderRejected);
        assert_eq!(
            failure.stage(),
            WorthQueryProviderSessionProtocolStage::Commit
        );
        assert_eq!(
            failure.recovery_posture(),
            WorthQueryProviderSessionRecoveryPosture::Closed
        );
        assert_eq!(
            failure.detail(),
            "consumed output currentness exceeded publication capacity"
        );
        assert_eq!(failure.output_verification_capacity_stop(), Some(capacity));
        assert_eq!(
            failure.clone().output_verification_capacity_stop(),
            Some(capacity)
        );
        assert!(failure.native_preparation_error().is_none());
        let denial = provider_session_denied(failure);
        assert_eq!(denial.kind(), Kind::ProviderRejected);
        assert_eq!(denial.stage(), Stage::ProviderCommit);
        assert_eq!(denial.output_verification_capacity_stop(), Some(capacity));
        assert!(denial.execution_denial_cause().is_none());
    }
}
