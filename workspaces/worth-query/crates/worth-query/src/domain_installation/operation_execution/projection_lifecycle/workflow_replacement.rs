use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryReplacementWitness;
use crate::runtime::WorthQueryWorkspace;

use super::cleanup_pending::{
    finish_transition, WorthQueryCleanupRetryCoreOutcome, WorthQueryCleanupRollbackCoreOutcome,
};
use super::operational_owner::WorthQueryOperationalProjection;
use super::transition_admission::open_transition_successor;
use super::transition_states::WorthQueryReplacedProjectionPhase;
use super::workflow_transition_states::{
    WorthQueryReplacedWorkflowProjection, WorthQueryReplacementCleanupPendingWorkflowProjection,
    WorthQueryWorkflowProjectionTransitionStop,
};
use super::{
    WorthQueryCurrentWorkflowProjection, WorthQueryLiveBoundWorkflowProjection,
    WorthQueryLiveProjectionReceipt, WorthQueryProjectionLifecycleCloseCause,
    WorthQueryProjectionLifecycleCloseReceipt, WorthQueryProjectionTransitionDenialKind,
    WorthQueryProjectionTransitionWork,
};

#[must_use = "workflow replacement outcomes retain every live resource on every branch"]
pub enum WorthQueryWorkflowProjectionReplacementOutcome<D, O, F, L: BasisOperationLane> {
    Replaced(WorthQueryReplacedWorkflowProjection<D, O, F, L>),
    Stopped(WorthQueryWorkflowProjectionTransitionStop<D, O, F, L>),
    CleanupPending(WorthQueryReplacementCleanupPendingWorkflowProjection<D, O, F, L>),
}

pub enum WorthQueryWorkflowReplacementCleanupRetryOutcome<D, O, F, L: BasisOperationLane> {
    Replaced(WorthQueryReplacedWorkflowProjection<D, O, F, L>),
    Pending(WorthQueryReplacementCleanupPendingWorkflowProjection<D, O, F, L>),
}

pub enum WorthQueryWorkflowReplacementRollbackOutcome<D, O, F, L: BasisOperationLane> {
    Restored {
        live: WorthQueryLiveBoundWorkflowProjection<D, O, F, L>,
        receipt: WorthQueryProjectionLifecycleCloseReceipt,
        work: super::WorthQueryProjectionCleanupWork,
    },
    Pending(WorthQueryReplacementCleanupPendingWorkflowProjection<D, O, F, L>),
}

impl<D: 'static, O, F, L: BasisOperationLane> WorthQueryLiveBoundWorkflowProjection<D, O, F, L> {
    pub fn replacement_witness_for(
        &self,
        candidate: &WorthQueryCurrentWorkflowProjection<D, O, F, L>,
    ) -> Result<WorthQueryReplacementWitness, crate::domain_installation::WorthQueryReplacementDenial>
    {
        self.snapshot()
            .bound_operation()
            .replacement_with(candidate.snapshot().bound_operation())
    }

    pub fn replace_with(
        self,
        candidate: WorthQueryCurrentWorkflowProjection<D, O, F, L>,
        witness: WorthQueryReplacementWitness,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryWorkflowProjectionReplacementOutcome<D, O, F, L> {
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
        candidate: WorthQueryCurrentWorkflowProjection<D, O, F, L>,
        witness: WorthQueryReplacementWitness,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryWorkflowProjectionReplacementOutcome<D, O, F, L> {
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
                "live workflow lifecycle proof no longer binds its retained projection",
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
                "workflow replacement predecessor is not current in this installed domain",
                work,
            );
        }
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
                    super::replacement::compatibility_kind(denial),
                    "replacement witness did not readmit for this exact workflow capability pair",
                    work,
                ),
            };
        let admitted = match open_transition_successor(
            execution,
            candidate,
            workspace,
            "worth_query_replaced_workflow_projection_v1",
            work,
        ) {
            Ok(admitted) => admitted,
            Err(stop) => {
                return WorthQueryWorkflowProjectionReplacementOutcome::Stopped(
                    WorthQueryWorkflowProjectionTransitionStop {
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
                WorthQueryWorkflowProjectionReplacementOutcome::Replaced(
                    WorthQueryReplacedWorkflowProjection {
                        transitioned,
                        witness,
                    },
                )
            }
            WorthQueryCleanupRetryCoreOutcome::Pending { pending, detail } => {
                WorthQueryWorkflowProjectionReplacementOutcome::CleanupPending(
                    WorthQueryReplacementCleanupPendingWorkflowProjection {
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
    WorthQueryReplacementCleanupPendingWorkflowProjection<D, O, F, L>
{
    pub fn retry_cleanup(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryWorkflowReplacementCleanupRetryOutcome<D, O, F, L> {
        let Self {
            pending, witness, ..
        } = self;
        match pending.retry(workspace) {
            WorthQueryCleanupRetryCoreOutcome::Completed(transitioned) => {
                WorthQueryWorkflowReplacementCleanupRetryOutcome::Replaced(
                    WorthQueryReplacedWorkflowProjection {
                        transitioned,
                        witness,
                    },
                )
            }
            WorthQueryCleanupRetryCoreOutcome::Pending { pending, detail } => {
                WorthQueryWorkflowReplacementCleanupRetryOutcome::Pending(Self {
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
    ) -> WorthQueryWorkflowReplacementRollbackOutcome<D, O, F, L> {
        let Self {
            pending, witness, ..
        } = self;
        match pending.rollback(workspace) {
            WorthQueryCleanupRollbackCoreOutcome::Restored {
                predecessor,
                rollback_close,
                work,
            } => WorthQueryWorkflowReplacementRollbackOutcome::Restored {
                live: WorthQueryLiveBoundWorkflowProjection::from_owner(predecessor),
                receipt: rollback_close,
                work,
            },
            WorthQueryCleanupRollbackCoreOutcome::Pending { pending, detail } => {
                WorthQueryWorkflowReplacementRollbackOutcome::Pending(Self {
                    pending,
                    detail,
                    witness,
                })
            }
        }
    }
}

fn stopped<D, O, F, L: BasisOperationLane>(
    live: WorthQueryLiveBoundWorkflowProjection<D, O, F, L>,
    candidate: WorthQueryCurrentWorkflowProjection<D, O, F, L>,
    kind: WorthQueryProjectionTransitionDenialKind,
    detail: &'static str,
    work: WorthQueryProjectionTransitionWork,
) -> WorthQueryWorkflowProjectionReplacementOutcome<D, O, F, L> {
    WorthQueryWorkflowProjectionReplacementOutcome::Stopped(
        WorthQueryWorkflowProjectionTransitionStop {
            live,
            candidate,
            kind,
            detail: detail.into(),
            work,
        },
    )
}
