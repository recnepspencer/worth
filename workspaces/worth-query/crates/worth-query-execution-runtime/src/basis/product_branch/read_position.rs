use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation, ProductBranchReferenceGeneration, ProductBranchReferenceSnapshot,
};

/// Declared World occurrence axes used to order conditional evaluations.
/// The owner-issued immutable commit determines its basis; the history owner
/// refuses duplicate commits. Exact admission tokens add no ordering axis.
///
/// Constructors take an owner-issued observation or reference snapshot. World
/// obtains commit and basis from one immutable commit; callers cannot splice
/// axes from different observations into this ordering position.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProductBranchReadPosition {
    branch: ProductBranchIdentity,
    incarnation: ProductBranchIncarnation,
    generation: ProductBranchReferenceGeneration,
    commit: CompositeCommitIdentity,
}

impl WorthQueryProductBranchReadPosition {
    pub(super) fn branch(&self) -> &ProductBranchIdentity {
        &self.branch
    }
    pub(super) fn incarnation(&self) -> ProductBranchIncarnation {
        self.incarnation
    }
    pub(super) fn generation(&self) -> ProductBranchReferenceGeneration {
        self.generation
    }
    pub(super) fn commit(&self) -> &CompositeCommitIdentity {
        &self.commit
    }

    pub(super) fn from_reference_snapshot(snapshot: &ProductBranchReferenceSnapshot) -> Self {
        Self {
            branch: snapshot.branch_identity().clone(),
            incarnation: snapshot.lifecycle_incarnation(),
            generation: snapshot.reference_generation(),
            commit: snapshot.selected_commit().clone(),
        }
    }

    pub(super) fn from_observation(observation: &ProductBranchObservation) -> Self {
        Self {
            branch: observation.branch_identity().clone(),
            incarnation: observation.lifecycle_incarnation(),
            generation: observation.reference_generation(),
            commit: observation.selected_commit().clone(),
        }
    }
}
