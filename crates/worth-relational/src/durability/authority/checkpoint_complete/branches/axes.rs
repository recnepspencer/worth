use super::*;

#[derive(Clone)]
pub(super) struct Axes {
    pub(super) generation: Generation,
    pub(super) truth: RelationalBranchVersion,
    pub(super) lifecycle: Lifecycle,
    pub(super) commit: Option<CommitId>,
    pub(super) schema: Arc<RelationalBranchRootSchemaAuthority>,
    pub(super) removed: bool,
}

impl Axes {
    pub(super) fn empty(original: &OriginalEmptySchemaObservation) -> Self {
        Self {
            generation: original.generation,
            truth: original.truth_version,
            lifecycle: original.lifecycle,
            commit: None,
            schema: original.schema.clone(),
            removed: false,
        }
    }
    pub(super) fn is_empty_observation(&self, original: &OriginalEmptySchemaObservation) -> bool {
        self.generation == original.generation
            && self.truth == original.truth_version
            && self.lifecycle == original.lifecycle
            && Arc::ptr_eq(&self.schema, &original.schema)
    }
    pub(super) fn matches(
        &self,
        generation: Generation,
        truth: RelationalBranchVersion,
        target: &FoundationalBranchTarget<crate::branch::RelationalBranchTarget>,
        roots: &FoldedRoots,
    ) -> bool {
        self.generation == generation && self.truth == truth && self.matches_target(target, roots)
    }
    pub(super) fn matches_basis(
        &self,
        target: &crate::branch::RelationalBranchTarget,
        roots: &FoldedRoots,
    ) -> bool {
        let Some(commit) = self.commit else {
            return false;
        };
        commit.0 == target.selected_commit_id()
            && roots.get(commit).is_some_and(|root| {
                root.root.descriptor() == Some(target.roots())
                    && root.root.commit_receipt().is_some_and(|receipt| {
                        receipt.version_id.0 == target.version_id()
                            && receipt
                                .parents
                                .iter()
                                .map(|id| id.0)
                                .eq(target.parent_commit_ids().iter().copied())
                    })
            })
    }
    pub(super) fn matches_target(
        &self,
        target: &FoundationalBranchTarget<crate::branch::RelationalBranchTarget>,
        roots: &FoldedRoots,
    ) -> bool {
        match (self.commit, target) {
            (None, FoundationalBranchTarget::Empty) => true,
            (Some(_), FoundationalBranchTarget::Basis(target)) => self.matches_basis(target, roots),
            _ => false,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct ObservationKey<'a> {
    branch: &'a BranchId,
    generation: u64,
    truth: u64,
    posture: u8,
    commit: Option<CommitId>,
}
impl<'a> ObservationKey<'a> {
    pub(super) fn of_axes(branch: &'a BranchId, axes: &Axes) -> Self {
        Self {
            branch,
            generation: axes.generation.get(),
            truth: axes.truth.as_u64(),
            posture: posture(axes.lifecycle),
            commit: axes.commit,
        }
    }
    pub(super) fn of_descriptor(descriptor: &'a RelationalBranchBasisDescriptor) -> Self {
        Self {
            branch: descriptor.branch_id(),
            generation: descriptor.reference().generation().get(),
            truth: descriptor.truth_version().as_u64(),
            posture: match descriptor.posture() {
                RelationalBranchBasisPosture::Live => 0,
                RelationalBranchBasisPosture::Archived => 1,
                RelationalBranchBasisPosture::Deleting => 2,
            },
            commit: match descriptor.reference().target() {
                FoundationalBranchTarget::Empty => None,
                FoundationalBranchTarget::Basis(target) => {
                    Some(CommitId(target.selected_commit_id()))
                }
            },
        }
    }
}

pub(super) enum RequiredSource<'a> {
    Canonical(&'a OriginalCanonicalCommit),
    Transition(&'a RetainedBranchTransition),
}
impl<'a> RequiredSource<'a> {
    pub(super) fn branch(&self) -> &'a BranchId {
        match self {
            Self::Canonical(commit) => &commit.envelope.branch_context,
            Self::Transition(entry) => match entry.positioned.transition() {
                RelationalBranchTransition::Fork { source, .. } => source,
                RelationalBranchTransition::Archive { .. }
                | RelationalBranchTransition::DeletionRequested { .. }
                | RelationalBranchTransition::Delete { .. }
                | RelationalBranchTransition::RetainedFork { .. } => {
                    entry.positioned.transition().branch()
                }
            },
        }
    }
    pub(super) fn matches(&self, axes: &Axes, roots: &FoldedRoots) -> bool {
        match self {
            Self::Canonical(commit) => {
                let Some(cell) = &commit.envelope.branch_cell_checkpoint else {
                    return false;
                };
                !axes.removed
                    && axes.lifecycle == Lifecycle::Live
                    && cell.branch_id == commit.envelope.branch_context
                    && axes.matches(
                        cell.observation.generation(),
                        cell.truth_version,
                        cell.observation.target(),
                        roots,
                    )
                    && Arc::ptr_eq(&axes.schema, &commit.inputs.prestate_schema)
            }
            Self::Transition(entry) => {
                if axes.removed || !Arc::ptr_eq(&axes.schema, &entry.schema) {
                    return false;
                }
                match entry.positioned.transition() {
                    RelationalBranchTransition::Fork {
                        source_generation,
                        source_truth_version,
                        source_head,
                        ..
                    } => {
                        axes.lifecycle == Lifecycle::Live
                            && axes.generation == *source_generation
                            && axes.truth == *source_truth_version
                            && axes.matches_basis(source_head, roots)
                    }
                    RelationalBranchTransition::Archive {
                        prior_generation,
                        prior_truth_version,
                        ..
                    } => {
                        axes.lifecycle == Lifecycle::Live
                            && axes.generation == *prior_generation
                            && axes.truth == *prior_truth_version
                    }
                    RelationalBranchTransition::DeletionRequested {
                        prior_generation,
                        prior_truth_version,
                        ..
                    } => axes.generation == *prior_generation && axes.truth == *prior_truth_version,
                    RelationalBranchTransition::Delete {
                        prior_generation,
                        prior_truth_version,
                        head_commit,
                        ..
                    } => {
                        axes.generation == *prior_generation
                            && axes.truth == *prior_truth_version
                            && axes.commit == *head_commit
                    }
                    RelationalBranchTransition::RetainedFork { .. } => false,
                }
            }
        }
    }
}
impl Axes {
    pub(super) fn same_state(&self, other: &Self) -> bool {
        self.generation == other.generation
            && self.truth == other.truth
            && self.lifecycle == other.lifecycle
            && self.commit == other.commit
            && self.removed == other.removed
            && Arc::ptr_eq(&self.schema, &other.schema)
    }
}
