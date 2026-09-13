use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::PhysicalArtifactScope;

use super::super::{RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection};
use super::{recorded, RecoveryIntegrityIngressAttempt};

pub(crate) fn observe_absent_recovery_artifact(
    observed: &ObservedRecoveryArtifact,
    expected_scope: PhysicalArtifactScope,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> RecoveryIntegrityIngressAttempt<'static> {
    let rejection = if observed.bytes().is_none() {
        RecoveryIntegrityIngressRejection::Absent
    } else {
        RecoveryIntegrityIngressRejection::SourceIncarnationMismatch
    };
    recorded(expected_scope, Err(rejection), counters)
}
