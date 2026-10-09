use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{WorthQueryCompatibilityUseDenial, WorthQueryReplacementWitness};
use crate::runtime::WorthQueryWorkspace;

use super::cleanup_pending::{
    finish_transition, WorthQueryCleanupRetryCoreOutcome, WorthQueryCleanupRollbackCoreOutcome,
};
use super::operational_owner::WorthQueryOperationalProjection;
use super::transition_admission::open_transition_successor;
use super::transition_states::{
    WorthQueryProjectionTransitionStop, WorthQueryReplacedDomainProjection,
    WorthQueryReplacedProjectionPhase, WorthQueryReplacementCleanupPendingDomainProjection,
};
use super::{
    WorthQueryCurrentDomainProjection, WorthQueryLiveBoundDomainProjection,
    WorthQueryLiveProjectionReceipt, WorthQueryProjectionLifecycleCloseCause,
    WorthQueryProjectionLifecycleCloseReceipt, WorthQueryProjectionTransitionDenialKind,
    WorthQueryProjectionTransitionWork,
};

#[must_use = "replacement outcomes retain every live resource on every branch"]
pub enum WorthQueryProjectionReplacementOutcome<D, O, F, L: BasisOperationLane> {
    Replaced(WorthQueryReplacedDomainProjection<D, O, F, L>),
    Stopped(WorthQueryProjectionTransitionStop<D, O, F, L>),
    CleanupPending(WorthQueryReplacementCleanupPendingDomainProjection<D, O, F, L>),
}

pub enum WorthQueryReplacementCleanupRetryOutcome<D, O, F, L: BasisOperationLane> {
    Replaced(WorthQueryReplacedDomainProjection<D, O, F, L>),
    Pending(WorthQueryReplacementCleanupPendingDomainProjection<D, O, F, L>),
}

pub enum WorthQueryReplacementRollbackOutcome<D, O, F, L: BasisOperationLane> {
    Restored {
        live: WorthQueryLiveBoundDomainProjection<D, O, F, L>,
        receipt: WorthQueryProjectionLifecycleCloseReceipt,
        work: super::WorthQueryProjectionCleanupWork,
    },
    Pending(WorthQueryReplacementCleanupPendingDomainProjection<D, O, F, L>),
}

impl<D: 'static, O, F, L: BasisOperationLane> WorthQueryLiveBoundDomainProjection<D, O, F, L> {
    pub fn replacement_witness_for(
        &self,
        candidate: &WorthQueryCurrentDomainProjection<D, O, F, L>,
    ) -> Result<WorthQueryReplacementWitness, crate::domain_installation::WorthQueryReplacementDenial>
    {
        self.snapshot()
            .bound_operation()
            .replacement_with(candidate.snapshot().bound_operation())
    }

    pub fn replace_with(
        self,
        candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
        witness: WorthQueryReplacementWitness,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryProjectionReplacementOutcome<D, O, F, L> {
        let owner = workspace.advancement_owner();
        let mut retained = Some((self, candidate, witness));
        match owner.with_advancement(|phase| {
            let (current, candidate, witness) = retained
                .take()
                .expect("host call retains its projection before admission");
            current.replace_with_in_advancement(
                phase
                    .execution_request_for(&owner)
                    .expect("the opener lent this owner its phase"),
                candidate,
                witness,
                workspace,
            )
        }) {
            Ok(outcome) => outcome,
            Err(denial) => {
                let (current, candidate, _witness) = retained
                    .take()
                    .expect("refused request ran no projection work");
                stopped(
                    current,
                    candidate,
                    WorthQueryProjectionTransitionDenialKind::ExecutionRequest(denial),
                    "advancement admission refused",
                    WorthQueryProjectionTransitionWork::new(),
                )
            }
        }
    }

    fn replace_with_in_advancement(
        self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
        witness: WorthQueryReplacementWitness,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryProjectionReplacementOutcome<D, O, F, L> {
        let mut work = WorthQueryProjectionTransitionWork::new();
        let mut checks = 0;
        if !self
            .owner
            .proof()
            .strong_basis()
            .value()
            .binds(self.snapshot(), &mut checks)
        {
            work.retain_authority_checks(checks);
            return stopped(
                self,
                candidate,
                WorthQueryProjectionTransitionDenialKind::BoundAuthorityMismatch,
                "live lifecycle proof no longer binds its retained projection",
                work,
            );
        }
        work.retain_authority_checks(checks + 1);
        if let Err(denial) =
            super::source::validate_live_source_authority(self.snapshot(), workspace)
        {
            return stopped(
                self,
                candidate,
                WorthQueryProjectionTransitionDenialKind::Authority(denial.kind()),
                "replacement predecessor is not current in this installed domain",
                work,
            );
        }
        work.retain_compatibility_readmission();
        let witness = match witness.readmit_for_pair(
            self.snapshot().bound_operation(),
            candidate.snapshot().bound_operation(),
        ) {
            Ok(witness) => witness,
            Err(denial) => {
                return stopped(
                    self,
                    candidate,
                    compatibility_kind(denial),
                    "replacement witness did not readmit for this exact capability pair",
                    work,
                )
            }
        };
        let admitted = match open_transition_successor(
            execution,
            candidate,
            workspace,
            "worth_query_replaced_projection_v1",
            work,
        ) {
            Ok(admitted) => admitted,
            Err(stop) => {
                return WorthQueryProjectionReplacementOutcome::Stopped(
                    WorthQueryProjectionTransitionStop {
                        live: self,
                        candidate: stop.current,
                        kind: stop.kind,
                        detail: stop.detail,
                        work: stop.work,
                    },
                )
            }
        };
        let predecessor_identity = self.identity().to_string();
        let settled_identity = admitted.current.snapshot().identity().to_string();
        let (settled, basis, _) = admitted.current.into_live_parts();
        let receipt = WorthQueryLiveProjectionReceipt::new(
            admitted.ready.operational_identity.clone(),
            admitted.ready.resource_name.clone(),
            settled_identity,
            admitted.ready.attempt,
            admitted.read_context_identity,
            admitted.ready.counters,
        );
        let successor =
            WorthQueryOperationalProjection::<_, L, WorthQueryReplacedProjectionPhase>::mint(
                settled,
                basis,
                predecessor_identity,
                admitted.ready.operational_identity,
                admitted.handle,
                receipt,
                admitted.ready.conditional_provenance,
            );
        match finish_transition(
            self.into_owner(),
            successor,
            WorthQueryProjectionLifecycleCloseCause::Replacement,
            WorthQueryProjectionLifecycleCloseCause::ReplacementRollback,
            admitted.work,
            workspace,
        ) {
            WorthQueryCleanupRetryCoreOutcome::Completed(transitioned) => {
                WorthQueryProjectionReplacementOutcome::Replaced(
                    WorthQueryReplacedDomainProjection {
                        transitioned,
                        witness,
                    },
                )
            }
            WorthQueryCleanupRetryCoreOutcome::Pending { pending, detail } => {
                WorthQueryProjectionReplacementOutcome::CleanupPending(
                    WorthQueryReplacementCleanupPendingDomainProjection {
                        pending,
                        detail,
                        witness,
                    },
                )
            }
        }
    }
}

impl<D, O, F, L: BasisOperationLane>
    WorthQueryReplacementCleanupPendingDomainProjection<D, O, F, L>
{
    pub fn retry_cleanup(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryReplacementCleanupRetryOutcome<D, O, F, L> {
        let WorthQueryReplacementCleanupPendingDomainProjection {
            pending, witness, ..
        } = self;
        match pending.retry(workspace) {
            WorthQueryCleanupRetryCoreOutcome::Completed(transitioned) => {
                WorthQueryReplacementCleanupRetryOutcome::Replaced(
                    WorthQueryReplacedDomainProjection {
                        transitioned,
                        witness,
                    },
                )
            }
            WorthQueryCleanupRetryCoreOutcome::Pending { pending, detail } => {
                WorthQueryReplacementCleanupRetryOutcome::Pending(Self {
                    pending,
                    detail,
                    witness,
                })
            }
        }
    }

    pub fn rollback(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryReplacementRollbackOutcome<D, O, F, L> {
        let WorthQueryReplacementCleanupPendingDomainProjection {
            pending, witness, ..
        } = self;
        match pending.rollback(workspace) {
            WorthQueryCleanupRollbackCoreOutcome::Restored {
                predecessor,
                rollback_close,
                work,
            } => WorthQueryReplacementRollbackOutcome::Restored {
                live: WorthQueryLiveBoundDomainProjection::from_owner(predecessor),
                receipt: rollback_close,
                work,
            },
            WorthQueryCleanupRollbackCoreOutcome::Pending { pending, detail } => {
                WorthQueryReplacementRollbackOutcome::Pending(Self {
                    pending,
                    detail,
                    witness,
                })
            }
        }
    }
}

fn stopped<D, O, F, L: BasisOperationLane>(
    live: WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
    kind: WorthQueryProjectionTransitionDenialKind,
    detail: &'static str,
    work: WorthQueryProjectionTransitionWork,
) -> WorthQueryProjectionReplacementOutcome<D, O, F, L> {
    WorthQueryProjectionReplacementOutcome::Stopped(WorthQueryProjectionTransitionStop {
        live,
        candidate,
        kind,
        detail: detail.into(),
        work,
    })
}

pub(super) fn compatibility_kind(
    denial: WorthQueryCompatibilityUseDenial,
) -> WorthQueryProjectionTransitionDenialKind {
    match denial {
        WorthQueryCompatibilityUseDenial::WrongCapabilityPair => {
            WorthQueryProjectionTransitionDenialKind::WrongCompatibilityPair
        }
        WorthQueryCompatibilityUseDenial::StaleAuthority => {
            WorthQueryProjectionTransitionDenialKind::StaleCompatibilityAuthority
        }
        WorthQueryCompatibilityUseDenial::StaleConditionalLowering => {
            WorthQueryProjectionTransitionDenialKind::StaleConditionalLowering
        }
    }
}
