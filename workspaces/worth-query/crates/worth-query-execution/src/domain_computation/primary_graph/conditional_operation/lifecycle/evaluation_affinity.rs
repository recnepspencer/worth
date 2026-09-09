use worth_runtime_bridge::facade::BridgeConditionalSignalBasisBinding;

/// Retained custody for one exact World-selected conditional evaluation.
///
/// The identity is an index key only. The retained product and Signal binding
/// remain the authority used by reconstruction and execution.
pub(in crate::domain_computation::primary_graph::conditional_operation) struct WorthQueryConditionalEvaluationAffinity
{
    identity: crate::basis::WorthQueryProductBranchReadIdentity,
    product: crate::basis::WorthQueryProductBranchLease,
    signal_basis: BridgeConditionalSignalBasisBinding,
}

impl WorthQueryConditionalEvaluationAffinity {
    pub(super) fn new(
        product: &crate::basis::WorthQueryProductBranchLease,
        signal_basis: BridgeConditionalSignalBasisBinding,
    ) -> Self {
        Self {
            identity: crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                product.observation(),
            ),
            product: product.retained_clone(),
            signal_basis,
        }
    }

    pub(super) fn identity(&self) -> &crate::basis::WorthQueryProductBranchReadIdentity {
        &self.identity
    }

    pub(super) fn is_predecessor_of(
        &self,
        candidate: &crate::basis::WorthQueryProductBranchReadIdentity,
    ) -> bool {
        self.identity.branch_identity() == candidate.branch_identity()
            && self.identity.lifecycle_incarnation() == candidate.lifecycle_incarnation()
            && self.identity < *candidate
    }

    pub(super) fn relational_commit_ceiling(
        &self,
    ) -> Option<worth_relational::facade::history::CommitId> {
        match self
            .product
            .relational_basis_descriptor()
            .reference()
            .target()
        {
            worth_foundational::facade::FoundationalBranchTarget::Empty => None,
            worth_foundational::facade::FoundationalBranchTarget::Basis(target) => Some(
                worth_relational::facade::history::CommitId(target.selected_commit_id()),
            ),
        }
    }

    pub(super) fn signal_basis(&self) -> &BridgeConditionalSignalBasisBinding {
        &self.signal_basis
    }
}
