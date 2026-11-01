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
                    && root.root.canonical_envelope().is_some_and(|body| {
                        body.commit.version_id.0 == target.version_id()
                            && body
                                .commit
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
