mod absence;
mod bootstrap;
mod checkpoint;
mod extent;
mod free_space;
mod page;
mod root;
mod segment_membership;
mod wal;

use super::admitted_artifact::IntegrityAdmittedRecoveryArtifact;
use super::observation::RecoveryIntegrityIngressObservation;
use super::{RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalIntegrityRejection};

pub(crate) struct RecoveryIntegrityIngressAttempt<'media> {
    outcome: Result<IntegrityAdmittedRecoveryArtifact<'media>, RecoveryIntegrityIngressRejection>,
    observation: RecoveryIntegrityIngressObservation,
}

pub(crate) use absence::observe_absent_recovery_artifact;

impl<'media> RecoveryIntegrityIngressAttempt<'media> {
    pub(crate) fn into_outcome(
        self,
    ) -> Result<IntegrityAdmittedRecoveryArtifact<'media>, RecoveryIntegrityIngressRejection> {
        self.outcome
    }

    pub(crate) const fn observation(&self) -> RecoveryIntegrityIngressObservation {
        self.observation
    }
}

pub(crate) fn rejected_source_binding<'media>(
    scope: PhysicalArtifactScope,
    rejection: RecoveryIntegrityIngressRejection,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> RecoveryIntegrityIngressAttempt<'media> {
    recorded(scope, Err(rejection), counters)
}

fn recorded<'media>(
    scope: PhysicalArtifactScope,
    outcome: Result<IntegrityAdmittedRecoveryArtifact<'media>, RecoveryIntegrityIngressRejection>,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> RecoveryIntegrityIngressAttempt<'media> {
    let observation = super::counters::record_admission(scope, &outcome, counters);
    RecoveryIntegrityIngressAttempt {
        outcome,
        observation,
    }
}

fn rejected_integrity<'media>(
    expected_scope: PhysicalArtifactScope,
    rejection: PhysicalIntegrityRejection,
    counters: &mut RecoveryIntegrityIngressCounters,
) -> RecoveryIntegrityIngressAttempt<'media> {
    let rejection = if rejection.scope() == expected_scope {
        RecoveryIntegrityIngressRejection::Integrity(rejection)
    } else {
        RecoveryIntegrityIngressRejection::ScopeMismatch
    };
    recorded(expected_scope, Err(rejection), counters)
}
