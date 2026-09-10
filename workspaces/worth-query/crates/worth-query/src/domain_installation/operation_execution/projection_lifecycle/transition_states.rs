use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryCurrentDomainProjection, WorthQuerySettledDomainProjection,
};
use worth_proof::PhaseMarker;

use super::cleanup_pending::{
    WorthQueryCleanupPendingCore, WorthQueryTransitionedOperationalProjection,
};
use super::{
    WorthQueryLiveBoundDomainProjection, WorthQueryProjectionCleanupWork,
    WorthQueryProjectionLifecycleCloseReceipt, WorthQueryProjectionTransitionDenialKind,
    WorthQueryProjectionTransitionWork,
};

pub(super) struct WorthQueryReplacedProjectionPhase;
pub(super) struct WorthQueryReboundProjectionPhase;
impl PhaseMarker for WorthQueryReplacedProjectionPhase {}
impl PhaseMarker for WorthQueryReboundProjectionPhase {}

#[must_use = "replaced projections own the admitted successor live resource"]
pub struct WorthQueryReplacedDomainProjection<D, O, F, L: BasisOperationLane> {
    pub(super) transitioned: WorthQueryTransitionedOperationalProjection<
        WorthQuerySettledDomainProjection<D, O, F, L>,
        L,
        WorthQueryReplacedProjectionPhase,
    >,
    pub(super) witness: crate::domain_installation::WorthQueryReplacementWitness,
}

#[must_use = "rebound projections own the admitted successor live resource"]
pub struct WorthQueryReboundDomainProjection<D, O, F, L: BasisOperationLane> {
    pub(super) transitioned: WorthQueryTransitionedOperationalProjection<
        WorthQuerySettledDomainProjection<D, O, F, L>,
        L,
        WorthQueryReboundProjectionPhase,
    >,
    pub(super) witness: crate::domain_installation::WorthQueryRebindWitness,
}

#[must_use = "cleanup-pending replacement owns both predecessor and successor resources"]
pub struct WorthQueryReplacementCleanupPendingDomainProjection<D, O, F, L: BasisOperationLane> {
    pub(super) pending: WorthQueryCleanupPendingCore<
        WorthQuerySettledDomainProjection<D, O, F, L>,
        WorthQuerySettledDomainProjection<D, O, F, L>,
        L,
        WorthQueryReplacedProjectionPhase,
    >,
    pub(super) detail: String,
    pub(super) witness: crate::domain_installation::WorthQueryReplacementWitness,
}

#[must_use = "cleanup-pending rebind owns both predecessor and successor resources"]
pub struct WorthQueryRebindCleanupPendingDomainProjection<D, O, F, L: BasisOperationLane> {
    pub(super) pending: WorthQueryCleanupPendingCore<
        WorthQuerySettledDomainProjection<D, O, F, L>,
        WorthQuerySettledDomainProjection<D, O, F, L>,
        L,
        WorthQueryReboundProjectionPhase,
    >,
    pub(super) detail: String,
    pub(super) witness: crate::domain_installation::WorthQueryRebindWitness,
}

pub struct WorthQueryProjectionTransitionStop<D, O, F, L: BasisOperationLane> {
    pub(super) live: WorthQueryLiveBoundDomainProjection<D, O, F, L>,
    pub(super) candidate: WorthQueryCurrentDomainProjection<D, O, F, L>,
    pub(super) kind: WorthQueryProjectionTransitionDenialKind,
    pub(super) detail: String,
    pub(super) work: WorthQueryProjectionTransitionWork,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryProjectionTransitionStop<D, O, F, L> {
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
        WorthQueryLiveBoundDomainProjection<D, O, F, L>,
        WorthQueryCurrentDomainProjection<D, O, F, L>,
    ) {
        (self.live, self.candidate)
    }
}

macro_rules! operational_inspection {
    ($name:ident) => {
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

            pub fn snapshot(&self) -> &WorthQuerySettledDomainProjection<D, O, F, L> {
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
        }
    };
}

operational_inspection!(WorthQueryReplacedDomainProjection);
operational_inspection!(WorthQueryReboundDomainProjection);

macro_rules! operational_refresh {
    ($name:ident) => {
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

operational_refresh!(WorthQueryReplacedDomainProjection);
operational_refresh!(WorthQueryReboundDomainProjection);

impl<D, O, F, L: BasisOperationLane> WorthQueryReplacedDomainProjection<D, O, F, L> {
    pub fn replacement_witness(&self) -> &crate::domain_installation::WorthQueryReplacementWitness {
        &self.witness
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryReboundDomainProjection<D, O, F, L> {
    pub fn rebind_witness(&self) -> &crate::domain_installation::WorthQueryRebindWitness {
        &self.witness
    }
}

impl<D, O, F, L: BasisOperationLane>
    WorthQueryReplacementCleanupPendingDomainProjection<D, O, F, L>
{
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

    pub fn replacement_witness(&self) -> &crate::domain_installation::WorthQueryReplacementWitness {
        &self.witness
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryRebindCleanupPendingDomainProjection<D, O, F, L> {
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

    pub fn rebind_witness(&self) -> &crate::domain_installation::WorthQueryRebindWitness {
        &self.witness
    }
}
