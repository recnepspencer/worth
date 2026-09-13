use std::cell::Cell;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::NonCurrentStagingBoundary;

use crate::{
    AuthorityAffectingRepairExecutionDenial, AuthorizationReplayPolicy,
    AuthorizationRevocationObservation, OperationalOperationId, OperationalTransitionId,
    RepairExecutionBoundary, RepairExecutionBoundaryMoment, RepairExecutionControlPort,
    RepairExecutionInterrupted, RepairExecutionInterruptionCause, RepairRecoveryDisposition,
    StoreOwnerKind,
};

use super::{
    certification_operator_assertion, repair_owner_recovery, CurrentScenarioStagingPort,
    ExactScenarioAuthorizationPort, OwnerBackedBackupScenario,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioRepairCancellationRecoveryReceipt {
    scheduler_cancellation_identity: [u8; 32],
    revocation_cancellation_identity: [u8; 32],
    backend_resume_identity: [u8; 32],
    evidence_identity: [u8; 32],
}

pub fn certify_scenario_repair_cancellation_recovery(
    case: &str,
) -> ScenarioRepairCancellationRecoveryReceipt {
    let scheduler_cancellation_identity = certify_scheduler_cancellation(case);
    let revocation_cancellation_identity = certify_revocation_cancellation(case);
    let backend_resume_identity = certify_backend_indeterminate_resume(case);
    let mut digest = Sha256::new();
    digest.update(b"worth-store-s10-repair-cancellation-recovery-v1");
    digest.update(scheduler_cancellation_identity);
    digest.update(revocation_cancellation_identity);
    digest.update(backend_resume_identity);
    ScenarioRepairCancellationRecoveryReceipt {
        scheduler_cancellation_identity,
        revocation_cancellation_identity,
        backend_resume_identity,
        evidence_identity: digest.finalize().into(),
    }
}

fn certify_scheduler_cancellation(case: &str) -> [u8; 32] {
    let name = format!("{case}/scheduler-denial");
    let scenario = OwnerBackedBackupScenario::materialize(&name);
    let control = scenario.control_store();
    let source = scenario.execute(&name, &control).into_restore_source();
    let scope = crate::OperationalSecurityScope::from_admission(source.custody().custody_receipt());
    let target = scenario.workspace_root().join("damaged.index");
    let replacement = scenario.workspace_root().join("replacement.index");
    std::fs::write(&target, b"damaged-derived-index").unwrap();
    std::fs::write(&replacement, b"rebuilt-derived-index").unwrap();
    let lowered = crate::workflow::certification_derived_maintenance_from_fixture_observation(
        OperationalOperationId::new(format!("{name}/repair")).unwrap(),
        &target,
        &replacement,
        scenario.authority().authority_identity(),
        scope,
    )
    .unwrap()
    .lower_owners()
    .unwrap();
    let restart = lowered.clone();
    let integrity = lowered
        .explanation()
        .nodes()
        .iter()
        .find(|node| node.owner() == StoreOwnerKind::PhysicalIntegrity)
        .expect("authority-affecting repair has a physical-integrity owner")
        .identity();
    let denial = lowered
        .authorize(
            &ExactScenarioAuthorizationPort,
            &certification_operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap()
        .ready(
            &control,
            OperationalTransitionId::new(format!("{name}/ready")).unwrap(),
            scenario.authority(),
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap()
        .execute_with_control(&DenyAt::scheduler(integrity));
    let crate::RepairExecutionDenial::Interrupted(interrupted) = denial.unwrap_err() else {
        panic!("scheduler denial must stop the repair at its declared boundary");
    };
    assert_eq!(
        interrupted.cause(),
        RepairExecutionInterruptionCause::SchedulerDenied
    );
    let handle = selected_handle(&scenario, &control);
    assert_eq!(
        handle.recovery_disposition(),
        RepairRecoveryDisposition::SafeToAbandonBeforeMutation
    );
    let stop = handle
        .abandon_before_mutation(&control, scenario.authority(), [0xa1; 32])
        .unwrap();
    assert_eq!(
        stop.disposition(),
        RepairRecoveryDisposition::SafeToAbandonBeforeMutation
    );
    assert!(repair_owner_recovery::repair_handles(&scenario, &control).is_empty());
    drop(restart);
    bind_interruption(interrupted, stop.basis())
}

fn certify_revocation_cancellation(case: &str) -> [u8; 32] {
    let name = format!("{case}/revocation");
    let repair_owner_recovery::RepairWorld {
        scenario,
        control,
        lowered,
    } = repair_owner_recovery::repair_world(&name);
    let denial = lowered
        .authorize(
            &ExactScenarioAuthorizationPort,
            &certification_operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap()
        .ready(
            &control,
            OperationalTransitionId::new(format!("{name}/ready")).unwrap(),
            scenario.authority(),
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap()
        .execute(&RevokeAtAllocation)
        .unwrap_err();
    let AuthorityAffectingRepairExecutionDenial::Authorization(
        crate::StagingAuthorizationContinuationDenial::Revoked { boundary },
    ) = denial
    else {
        panic!("revoked continuation must be preserved as the execution denial");
    };
    assert_eq!(boundary, NonCurrentStagingBoundary::Allocation);
    let handle = selected_handle(&scenario, &control);
    let disposition = handle.recovery_disposition();
    assert!(matches!(
        disposition,
        RepairRecoveryDisposition::NonCurrentResidueRemainsIsolated { .. }
    ));
    let stop = handle
        .retain_isolated_non_current_residue(&control, scenario.authority(), [0xa2; 32])
        .unwrap();
    assert_eq!(stop.disposition(), disposition);
    assert!(repair_owner_recovery::repair_handles(&scenario, &control).is_empty());
    let mut digest = Sha256::new();
    digest.update(b"revoked-at-allocation");
    digest.update(stop.basis());
    digest.update(handle.plan_fingerprint());
    digest.finalize().into()
}

fn certify_backend_indeterminate_resume(case: &str) -> [u8; 32] {
    let name = format!("{case}/backend-indeterminate");
    let repair_owner_recovery::RepairWorld {
        scenario,
        control,
        lowered,
    } = repair_owner_recovery::repair_world(&name);
    let backend = lowered
        .explanation()
        .nodes()
        .iter()
        .find(|node| node.owner() == StoreOwnerKind::PhysicalBackend)
        .unwrap()
        .identity();
    let restart = lowered.clone();
    let denial = lowered
        .authorize(
            &ExactScenarioAuthorizationPort,
            &certification_operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap()
        .ready(
            &control,
            OperationalTransitionId::new(format!("{name}/ready")).unwrap(),
            scenario.authority(),
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap()
        .execute_with_control(&CurrentScenarioStagingPort, &DenyAt::backend(backend));
    let AuthorityAffectingRepairExecutionDenial::Interrupted(interrupted) = denial.unwrap_err()
    else {
        panic!("backend indeterminacy must stop at the durable-effect boundary");
    };
    assert_eq!(
        interrupted.cause(),
        RepairExecutionInterruptionCause::BackendIndeterminate
    );
    let handle = selected_handle(&scenario, &control);
    assert!(matches!(
        handle.recovery_disposition(),
        RepairRecoveryDisposition::NonCurrentResidueRemainsIsolated { durable_owner_effects } if durable_owner_effects > 0
    ));
    let executed = restart
        .recover_ready(&handle, &control, scenario.authority())
        .unwrap()
        .execute(&CurrentScenarioStagingPort)
        .unwrap();
    assert!(repair_owner_recovery::repair_handles(&scenario, &control).is_empty());
    let mut digest = Sha256::new();
    digest.update(bind_interruption(interrupted, handle.plan_fingerprint()));
    digest.update(executed.staged_media().content_fingerprint());
    digest.finalize().into()
}

fn selected_handle(
    scenario: &OwnerBackedBackupScenario,
    control: &crate::OperationalControlStore,
) -> crate::IndeterminateRepairRecoveryHandle {
    repair_owner_recovery::repair_handles(scenario, control)
        .into_iter()
        .next()
        .expect("interrupted repair recovery handle")
}

fn bind_interruption(interruption: RepairExecutionInterrupted, basis: [u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(interruption.boundary().node().fingerprint());
    digest.update([interruption.cause() as u8]);
    digest.update(basis);
    digest.finalize().into()
}

struct DenyAt {
    node: crate::OwnerPlanNodeIdentity,
    moment: RepairExecutionBoundaryMoment,
    cause: RepairExecutionInterruptionCause,
    fired: Cell<bool>,
}

impl DenyAt {
    const fn scheduler(node: crate::OwnerPlanNodeIdentity) -> Self {
        Self::new(
            node,
            RepairExecutionBoundaryMoment::BeforeOwnerEffect,
            RepairExecutionInterruptionCause::SchedulerDenied,
        )
    }
    const fn backend(node: crate::OwnerPlanNodeIdentity) -> Self {
        Self::new(
            node,
            RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
            RepairExecutionInterruptionCause::BackendIndeterminate,
        )
    }
    const fn new(
        node: crate::OwnerPlanNodeIdentity,
        moment: RepairExecutionBoundaryMoment,
        cause: RepairExecutionInterruptionCause,
    ) -> Self {
        Self {
            node,
            moment,
            cause,
            fired: Cell::new(false),
        }
    }
}

impl RepairExecutionControlPort for DenyAt {
    fn observe(&self, boundary: RepairExecutionBoundary) -> Result<(), RepairExecutionInterrupted> {
        if !self.fired.get() && boundary.node() == self.node && boundary.moment() == self.moment {
            self.fired.set(true);
            let denial = match self.cause {
                RepairExecutionInterruptionCause::SchedulerDenied => {
                    RepairExecutionInterrupted::scheduler_denied(boundary)
                }
                RepairExecutionInterruptionCause::BackendIndeterminate => {
                    RepairExecutionInterrupted::backend_indeterminate(boundary)
                }
                RepairExecutionInterruptionCause::ProcessLoss => {
                    RepairExecutionInterrupted::at(boundary)
                }
            };
            Err(denial)
        } else {
            Ok(())
        }
    }
}

struct RevokeAtAllocation;

impl crate::StagingAuthorizationContinuationPort for RevokeAtAllocation {
    fn observe_revocation(
        &self,
        request: crate::StagingAuthorizationContinuationRequest,
    ) -> Result<AuthorizationRevocationObservation, crate::AuthorizationProviderFailure> {
        if request.boundary() == NonCurrentStagingBoundary::Allocation {
            Ok(AuthorizationRevocationObservation::Revoked {
                observed_at: 40,
                reason_fingerprint: [0xa3; 32],
            })
        } else {
            Ok(AuthorizationRevocationObservation::NotRevoked { observed_at: 40 })
        }
    }
}

impl crate::workflow::StagedWalApplicationPort for RevokeAtAllocation {
    fn apply_staged_wal(
        &self,
        request: crate::workflow::StagedWalApplicationRequest<'_>,
    ) -> Result<
        crate::workflow::StagedWalApplicationProviderReceipt,
        crate::workflow::StagedWalApplicationDenial,
    > {
        CurrentScenarioStagingPort.apply_staged_wal(request)
    }
}

impl ScenarioRepairCancellationRecoveryReceipt {
    pub const fn scheduler_cancellation_identity(self) -> [u8; 32] {
        self.scheduler_cancellation_identity
    }
    pub const fn revocation_cancellation_identity(self) -> [u8; 32] {
        self.revocation_cancellation_identity
    }
    pub const fn backend_resume_identity(self) -> [u8; 32] {
        self.backend_resume_identity
    }
    pub const fn evidence_identity(self) -> [u8; 32] {
        self.evidence_identity
    }
}
