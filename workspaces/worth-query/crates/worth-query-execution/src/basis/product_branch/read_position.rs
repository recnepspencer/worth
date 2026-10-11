use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchObservation, ProductBranchReferenceGeneration,
};

/// Declared World occurrence axes used to order conditional evaluations.
/// The owner-issued immutable commit determines its basis; the history owner
/// refuses duplicate commits. Exact admission tokens add no ordering axis.
///
/// The only constructor takes an owner-issued observation. The World snapshot
/// obtains commit and basis from one immutable commit; callers cannot splice
/// axes from different observations into this ordering position.
///
/// ```
/// mod position {
///     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/basis/product_branch/read_position.rs"));
/// }
/// fn observe(o: &worth_runtime_world::facade::ProductBranchObservation) {
///     let _ = position::WorthQueryProductBranchReadPosition::from_observation(o);
/// }
/// ```
///
/// ```compile_fail,E0451
/// mod position {
///     include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/basis/product_branch/read_position.rs"));
/// }
/// fn splice(a: &worth_runtime_world::facade::ProductBranchObservation,
///           b: &worth_runtime_world::facade::ProductBranchObservation) {
///     let _ = position::WorthQueryProductBranchReadPosition {
///         branch: a.branch_identity().clone(),
///         incarnation: a.lifecycle_incarnation(),
///         generation: a.reference_generation(),
///         commit: b.selected_commit().clone(),
///     };
/// }
/// ```
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct WorthQueryProductBranchReadPosition {
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

    pub(super) fn from_observation(observation: &ProductBranchObservation) -> Self {
        Self {
            branch: observation.branch_identity().clone(),
            incarnation: observation.lifecycle_incarnation(),
            generation: observation.reference_generation(),
            commit: observation.selected_commit().clone(),
        }
    }
}
