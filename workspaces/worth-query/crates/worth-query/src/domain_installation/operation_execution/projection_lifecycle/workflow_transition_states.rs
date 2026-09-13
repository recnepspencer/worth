use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryCurrentWorkflowProjection, WorthQuerySettledWorkflowProjection,
};

use super::cleanup_pending::{
    WorthQueryCleanupPendingCore, WorthQueryTransitionedOperationalProjection,
};
use super::transition_states::{
    WorthQueryReboundProjectionPhase, WorthQueryReplacedProjectionPhase,
};
use super::{
    WorthQueryLiveBoundWorkflowProjection, WorthQueryProjectionCleanupWork,
    WorthQueryProjectionLifecycleCloseReceipt, WorthQueryProjectionTransitionDenialKind,
    WorthQueryProjectionTransitionWork,
};

#[must_use = "replaced workflow projections own the admitted successor live resource"]
pub struct WorthQueryReplacedWorkflowProjection<D, O, F, L: BasisOperationLane> {
    pub(super) transitioned: WorthQueryTransitionedOperationalProjection<
        WorthQuerySettledWorkflowProjection<D, O, F, L>,
        L,
        WorthQueryReplacedProjectionPhase,
    >,
    pub(super) witness: crate::domain_installation::WorthQueryReplacementWitness,
}

#[must_use = "rebound workflow projections own the admitted successor live resource"]
pub struct WorthQueryReboundWorkflowProjection<D, O, F, L: BasisOperationLane> {
    pub(super) transitioned: WorthQueryTransitionedOperationalProjection<
        WorthQuerySettledWorkflowProjection<D, O, F, L>,
        L,
        WorthQueryReboundProjectionPhase,
    >,
    pub(super) witness: crate::domain_installation::WorthQueryRebindWitness,
}

#[must_use = "cleanup-pending workflow replacement owns both managed resources"]
pub struct WorthQueryReplacementCleanupPendingWorkflowProjection<D, O, F, L: BasisOperationLane> {
    pub(super) pending: WorthQueryCleanupPendingCore<
        WorthQuerySettledWorkflowProjection<D, O, F, L>,
        WorthQuerySettledWorkflowProjection<D, O, F, L>,
        L,
        WorthQueryReplacedProjectionPhase,
    >,
    pub(super) detail: String,
    pub(super) witness: crate::domain_installation::WorthQueryReplacementWitness,
}

#[must_use = "cleanup-pending workflow rebind owns both managed resources"]
pub struct WorthQueryRebindCleanupPendingWorkflowProjection<D, O, F, L: BasisOperationLane> {
    pub(super) pending: WorthQueryCleanupPendingCore<
        WorthQuerySettledWorkflowProjection<D, O, F, L>,
        WorthQuerySettledWorkflowProjection<D, O, F, L>,
        L,
        WorthQueryReboundProjectionPhase,
    >,
    pub(super) detail: String,
    pub(super) witness: crate::domain_installation::WorthQueryRebindWitness,
}

pub struct WorthQueryWorkflowProjectionTransitionStop<D, O, F, L: BasisOperationLane> {
    pub(super) live: WorthQueryLiveBoundWorkflowProjection<D, O, F, L>,
    pub(super) candidate: WorthQueryCurrentWorkflowProjection<D, O, F, L>,
    pub(super) kind: WorthQueryProjectionTransitionDenialKind,
    pub(super) detail: String,
    pub(super) work: WorthQueryProjectionTransitionWork,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryWorkflowProjectionTransitionStop<D, O, F, L> {
    pub fn kind(&self) -> WorthQueryProjectionTransitionDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn work(&self) -> WorthQueryProjectionTransitionWork {
        self.work
    }

    pub fn into_retry_parts(
        self,
    ) -> (
        WorthQueryLiveBoundWorkflowProjection<D, O, F, L>,
        WorthQueryCurrentWorkflowProjection<D, O, F, L>,
    ) {
        (self.live, self.candidate)
    }
}

macro_rules! operational_inspection {
    ($name:ident, $witness_method:ident, $witness:ty) => {
        impl<D, O, F, L: BasisOperationLane> $name<D, O, F, L> {
            pub fn identity(&self) -> &str {
                &self.transitioned.successor().proof().payload().identity
            }

            pub fn predecessor_identity(&self) -> &str {
                &self
                    .transitioned
                    .successor()
                    .proof()
                    .payload()
                    .predecessor_identity
            }

            pub fn resource_name(&self) -> &str {
                self.transitioned.successor().handle().name()
            }

            pub fn snapshot(&self) -> &WorthQuerySettledWorkflowProjection<D, O, F, L> {
                self.transitioned.successor().source()
            }

            pub fn conditional_provenance(
                &self,
            ) -> &[crate::domain_installation::WorthQueryConditionalProvenance] {
                self.transitioned.successor().conditional_provenance()
            }

            pub fn predecessor_close_receipt(&self) -> &WorthQueryProjectionLifecycleCloseReceipt {
                self.transitioned.predecessor_close()
            }

            pub fn transition_work(&self) -> WorthQueryProjectionTransitionWork {
                self.transitioned.work()
            }

            pub fn cleanup_work(&self) -> WorthQueryProjectionCleanupWork {
                self.transitioned.cleanup_work()
            }

            pub fn $witness_method(&self) -> &$witness {
                &self.witness
            }
        }

        impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> $name<D, O, F, L> {
            pub fn refresh(
                &self,
                workspace: &mut crate::runtime::WorthQueryWorkspace,
            ) -> Result<
                super::WorthQueryLiveProjectionRefresh,
                super::WorthQueryLiveProjectionRefreshError,
            > {
                super::refresh::refresh_source(
                    self.transitioned.successor().source(),
                    self.transitioned.successor().handle(),
                    workspace,
                )
            }
        }
    };
}

operational_inspection!(
    WorthQueryReplacedWorkflowProjection,
    replacement_witness,
    crate::domain_installation::WorthQueryReplacementWitness
);
operational_inspection!(
    WorthQueryReboundWorkflowProjection,
    rebind_witness,
    crate::domain_installation::WorthQueryRebindWitness
);

macro_rules! pending_inspection {
    ($name:ident, $witness_method:ident, $witness:ty) => {
        impl<D, O, F, L: BasisOperationLane> $name<D, O, F, L> {
            pub fn detail(&self) -> &str {
                &self.detail
            }

            pub fn transition_work(&self) -> WorthQueryProjectionTransitionWork {
                self.pending.work()
            }

            pub fn cleanup_work(&self) -> WorthQueryProjectionCleanupWork {
                self.pending.cleanup_work()
            }

            pub fn predecessor_resource_name(&self) -> &str {
                self.pending.predecessor_resource_name()
            }

            pub fn successor_resource_name(&self) -> &str {
                self.pending.successor_resource_name()
            }

            pub fn $witness_method(&self) -> &$witness {
                &self.witness
            }
        }
    };
}

pending_inspection!(
    WorthQueryReplacementCleanupPendingWorkflowProjection,
    replacement_witness,
    crate::domain_installation::WorthQueryReplacementWitness
);
pending_inspection!(
    WorthQueryRebindCleanupPendingWorkflowProjection,
    rebind_witness,
    crate::domain_installation::WorthQueryRebindWitness
);
