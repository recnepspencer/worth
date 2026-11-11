use crate::basis_lifecycle::BasisOperationLane;
use crate::ordinary::live::WorthQueryManagedLiveHandle;
use worth_proof::{Artifact, CurrentValidity, FreshnessScopedBasis, NoProofs, PhaseMarker};

use super::states::{
    lifecycle_authority, WorthQueryProjectionLifecycleBasis, WorthQueryProjectionLifecycleEvidence,
};
use super::WorthQueryLiveProjectionReceipt;

pub(in crate::domain_installation::operation_execution) type WorthQueryOperationalProjectionProof<
    P,
    L,
> = Artifact<
    P,
    WorthQueryProjectionLifecycleEvidence,
    NoProofs,
    FreshnessScopedBasis<
        CurrentValidity,
        worth_proof::AssumptionBasis<WorthQueryProjectionLifecycleBasis<L>>,
    >,
>;

/// The sole private owner of an operational lifecycle resource.
///
/// Public direct and workflow states wrap this type rather than reproducing
/// handle ownership or transition mechanics.
pub(in crate::domain_installation::operation_execution) struct WorthQueryOperationalProjection<
    S,
    L: BasisOperationLane,
    P: PhaseMarker,
> {
    source: S,
    proof: WorthQueryOperationalProjectionProof<P, L>,
    handle: WorthQueryManagedLiveHandle,
    receipt: WorthQueryLiveProjectionReceipt,
    conditional_provenance: Vec<super::super::super::WorthQueryConditionalProvenance>,
}

impl<S, L: BasisOperationLane, P: PhaseMarker> WorthQueryOperationalProjection<S, L, P> {
    pub(super) fn mint(
        source: S,
        basis: WorthQueryProjectionLifecycleBasis<L>,
        predecessor_identity: String,
        identity: String,
        handle: WorthQueryManagedLiveHandle,
        receipt: WorthQueryLiveProjectionReceipt,
        conditional_provenance: Vec<super::super::super::WorthQueryConditionalProvenance>,
    ) -> Self {
        let proof = Artifact::with_current_basis(
            WorthQueryProjectionLifecycleEvidence {
                identity,
                predecessor_identity,
            },
            basis,
            lifecycle_authority(),
        );
        Self {
            source,
            proof,
            handle,
            receipt,
            conditional_provenance,
        }
    }

    pub(super) fn source(&self) -> &S {
        &self.source
    }

    pub(super) fn proof(&self) -> &WorthQueryOperationalProjectionProof<P, L> {
        &self.proof
    }

    pub(super) fn handle(&self) -> &WorthQueryManagedLiveHandle {
        &self.handle
    }

    pub(super) fn receipt(&self) -> &WorthQueryLiveProjectionReceipt {
        &self.receipt
    }

    pub(super) fn conditional_provenance(
        &self,
    ) -> &[super::super::super::WorthQueryConditionalProvenance] {
        &self.conditional_provenance
    }

    pub(in crate::domain_installation::operation_execution) fn into_parts(
        self,
    ) -> (
        S,
        WorthQueryOperationalProjectionProof<P, L>,
        WorthQueryManagedLiveHandle,
        WorthQueryLiveProjectionReceipt,
        Vec<super::super::super::WorthQueryConditionalProvenance>,
    ) {
        (
            self.source,
            self.proof,
            self.handle,
            self.receipt,
            self.conditional_provenance,
        )
    }

    pub(in crate::domain_installation::operation_execution) fn from_parts(
        source: S,
        proof: WorthQueryOperationalProjectionProof<P, L>,
        handle: WorthQueryManagedLiveHandle,
        receipt: WorthQueryLiveProjectionReceipt,
        conditional_provenance: Vec<super::super::super::WorthQueryConditionalProvenance>,
    ) -> Self {
        Self {
            source,
            proof,
            handle,
            receipt,
            conditional_provenance,
        }
    }
}
#[cfg(test)]
#[path = "operational_owner_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
