use worth_runtime_world::facade::{
    CompositeBasisKey, CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation, ProductBranchReferenceGeneration,
};

/// Describes the exact World occurrence used by a read. It carries no live
/// admission or publication authority and retains no owner resources.
/// Equality preserves every observation axis: World validates that the
/// snapshot owner equals the owner embedded in its branch, incarnation,
/// commit and basis when issuing `ProductBranchReferenceSnapshot`.
///
/// ```compile_fail,E0277
/// fn ordered<T: Ord>() {}
/// ordered::<worth_query_execution::facade::product::WorthQueryProductBranchReadIdentity>();
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct WorthQueryProductBranchReadIdentity {
    position: super::read_position::WorthQueryProductBranchReadPosition,
    basis: CompositeBasisKey,
}

impl std::fmt::Debug for WorthQueryProductBranchReadIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryProductBranchReadIdentity")
            .field("branch", &self.position.branch)
            .field("incarnation", &self.position.incarnation)
            .field("generation", &self.position.generation)
            .field("commit", &self.position.commit)
            .field("basis", &self.basis)
            .finish()
    }
}

impl WorthQueryProductBranchReadIdentity {
    pub(crate) fn position(&self) -> &super::read_position::WorthQueryProductBranchReadPosition {
        &self.position
    }

    pub(crate) fn matches_observation(&self, observation: &ProductBranchObservation) -> bool {
        self.same_branch_occurrence_observation(observation)
            && self.position.generation == observation.reference_generation()
            && &self.position.commit == observation.selected_commit()
            && &self.basis == observation.basis().identity()
    }

    pub(crate) fn same_branch_occurrence_observation(
        &self,
        observation: &ProductBranchObservation,
    ) -> bool {
        &self.position.branch == observation.branch_identity()
            && self.position.incarnation == observation.lifecycle_incarnation()
    }

    pub(crate) fn from_observation(observation: &ProductBranchObservation) -> Self {
        Self {
            position: super::read_position::WorthQueryProductBranchReadPosition::from_observation(
                observation,
            ),
            basis: observation.basis().identity().clone(),
        }
    }

    pub fn branch_identity(&self) -> &ProductBranchIdentity {
        &self.position.branch
    }
    pub fn product_branch(&self) -> super::WorthQueryProductBranch {
        super::WorthQueryProductBranch::from_occurrence(self.position.incarnation)
    }
    pub fn lifecycle_incarnation(&self) -> ProductBranchIncarnation {
        self.position.incarnation
    }
    pub fn reference_generation(&self) -> ProductBranchReferenceGeneration {
        self.position.generation
    }
    pub fn selected_commit(&self) -> &CompositeCommitIdentity {
        &self.position.commit
    }
    pub fn composite_basis_identity(&self) -> &CompositeBasisKey {
        &self.basis
    }

    pub(crate) fn same_branch_occurrence(&self, other: &Self) -> bool {
        self.position.branch == other.position.branch
            && self.position.incarnation == other.position.incarnation
    }
}
