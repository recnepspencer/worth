use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryRebindWitness;
use crate::runtime::WorthQueryWorkspace;

use super::cleanup_pending::{
    finish_transition, WorthQueryCleanupRetryCoreOutcome, WorthQueryCleanupRollbackCoreOutcome,
};
use super::operational_owner::WorthQueryOperationalProjection;
use super::replacement::compatibility_kind;
use super::transition_admission::open_transition_successor;
use super::transition_states::{
    WorthQueryProjectionTransitionStop, WorthQueryRebindCleanupPendingDomainProjection,
    WorthQueryReboundDomainProjection, WorthQueryReboundProjectionPhase,
};
use super::{
    WorthQueryCurrentDomainProjection, WorthQueryLiveBoundDomainProjection,
    WorthQueryLiveProjectionReceipt, WorthQueryProjectionLifecycleCloseCause,
    WorthQueryProjectionLifecycleCloseReceipt, WorthQueryProjectionTransitionDenialKind,
    WorthQueryProjectionTransitionWork,
};

#[must_use = "rebind outcomes retain every live resource on every branch"]
pub enum WorthQueryProjectionRebindOutcome<D, O, F, L: BasisOperationLane> {
    Rebound(WorthQueryReboundDomainProjection<D, O, F, L>),
    Stopped(WorthQueryProjectionTransitionStop<D, O, F, L>),
    CleanupPending(WorthQueryRebindCleanupPendingDomainProjection<D, O, F, L>),
}

pub enum WorthQueryRebindCleanupRetryOutcome<D, O, F, L: BasisOperationLane> {
    Rebound(WorthQueryReboundDomainProjection<D, O, F, L>),
    Pending(WorthQueryRebindCleanupPendingDomainProjection<D, O, F, L>),
}

pub enum WorthQueryRebindRollbackOutcome<D, O, F, L: BasisOperationLane> {
    Restored {
        live: WorthQueryLiveBoundDomainProjection<D, O, F, L>,
        receipt: WorthQueryProjectionLifecycleCloseReceipt,
        work: super::WorthQueryProjectionCleanupWork,
    },
    Pending(WorthQueryRebindCleanupPendingDomainProjection<D, O, F, L>),
}

impl<D: 'static, O, F, L: BasisOperationLane> WorthQueryLiveBoundDomainProjection<D, O, F, L> {
    pub fn rebind_witness_for(
        &self,
        candidate: &WorthQueryCurrentDomainProjection<D, O, F, L>,
        receipt: crate::domain_installation::WorthQueryDomainRebindReceipt,
    ) -> Result<
        WorthQueryRebindWitness,
        crate::domain_installation::WorthQueryRebindCompatibilityDenial,
    > {
        self.snapshot()
            .bound_operation()
            .rebind_with(candidate.snapshot().bound_operation(), receipt)
    }

    pub fn rebind_witness_for_with_required_domains(
        &self,
        candidate: &WorthQueryCurrentDomainProjection<D, O, F, L>,
        receipt: crate::domain_installation::WorthQueryDomainRebindReceipt,
        required_domain_receipts: Vec<crate::domain_installation::WorthQueryDomainRebindReceipt>,
    ) -> Result<
        WorthQueryRebindWitness,
        crate::domain_installation::WorthQueryRebindCompatibilityDenial,
    > {
        self.snapshot()
            .bound_operation()
            .rebind_with_required_domain_receipts(
                candidate.snapshot().bound_operation(),
                receipt,
                required_domain_receipts,
            )
    }

    pub fn rebind_with(
        self,
        candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
        witness: WorthQueryRebindWitness,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryProjectionRebindOutcome<D, O, F, L> {
        let owner = workspace.advancement_owner();
        let mut retained = Some((self, candidate, witness));
        match owner.with_advancement(|phase| {
            let (current, candidate, witness) = retained
                .take()
                .expect("host call retains its projection before admission");
            current.rebind_with_in_advancement(
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

    fn rebind_with_in_advancement(
        self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
        witness: WorthQueryRebindWitness,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryProjectionRebindOutcome<D, O, F, L> {
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
        work.retain_authority_checks(checks);
        work.retain_compatibility_readmission();
        let witness =
            match witness.readmit_for_pair(
                self.snapshot().bound_operation(),
                candidate.snapshot().bound_operation(),
            ) {
                Ok(witness) => witness,
                Err(denial) => return stopped(
                    self,
                    candidate,
                    compatibility_kind(denial),
                    "rebind witness did not readmit for this exact stale/current capability pair",
                    work,
                ),
            };
        let admitted = match open_transition_successor(
            execution,
            candidate,
            workspace,
            "worth_query_rebound_projection_v1",
            work,
        ) {
            Ok(admitted) => admitted,
            Err(stop) => {
                return WorthQueryProjectionRebindOutcome::Stopped(
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
            WorthQueryOperationalProjection::<_, L, WorthQueryReboundProjectionPhase>::mint(
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
            WorthQueryProjectionLifecycleCloseCause::Rebind,
            WorthQueryProjectionLifecycleCloseCause::RebindRollback,
            admitted.work,
            workspace,
        ) {
            WorthQueryCleanupRetryCoreOutcome::Completed(transitioned) => {
                WorthQueryProjectionRebindOutcome::Rebound(WorthQueryReboundDomainProjection {
                    transitioned,
                    witness,
                })
            }
            WorthQueryCleanupRetryCoreOutcome::Pending { pending, detail } => {
                WorthQueryProjectionRebindOutcome::CleanupPending(
                    WorthQueryRebindCleanupPendingDomainProjection {
                        pending,
                        detail,
                        witness,
                    },
                )
            }
        }
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryRebindCleanupPendingDomainProjection<D, O, F, L> {
    pub fn retry_cleanup(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryRebindCleanupRetryOutcome<D, O, F, L> {
        let WorthQueryRebindCleanupPendingDomainProjection {
            pending, witness, ..
        } = self;
        match pending.retry(workspace) {
            WorthQueryCleanupRetryCoreOutcome::Completed(transitioned) => {
                WorthQueryRebindCleanupRetryOutcome::Rebound(WorthQueryReboundDomainProjection {
                    transitioned,
                    witness,
                })
            }
            WorthQueryCleanupRetryCoreOutcome::Pending { pending, detail } => {
                WorthQueryRebindCleanupRetryOutcome::Pending(Self {
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
    ) -> WorthQueryRebindRollbackOutcome<D, O, F, L> {
        let WorthQueryRebindCleanupPendingDomainProjection {
            pending, witness, ..
        } = self;
        match pending.rollback(workspace) {
            WorthQueryCleanupRollbackCoreOutcome::Restored {
                predecessor,
                rollback_close,
                work,
            } => WorthQueryRebindRollbackOutcome::Restored {
                live: WorthQueryLiveBoundDomainProjection::from_owner(predecessor),
                receipt: rollback_close,
                work,
            },
            WorthQueryCleanupRollbackCoreOutcome::Pending { pending, detail } => {
                WorthQueryRebindRollbackOutcome::Pending(Self {
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
) -> WorthQueryProjectionRebindOutcome<D, O, F, L> {
    WorthQueryProjectionRebindOutcome::Stopped(WorthQueryProjectionTransitionStop {
        live,
        candidate,
        kind,
        detail: detail.into(),
        work,
    })
}
