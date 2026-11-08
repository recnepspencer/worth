//! Install Native generation bindings and retire replaced identities.
use super::*;

impl GenerationCatalog {
    pub(in crate::runtime::state::subsystems::indexing) fn publish(
        &mut self,
        generation: DerivedIndexGeneration,
    ) {
        assert!(
            /* Proof: Only native generation publication calls publish. Generation identities are allocated by the checked, non-wrapping owner issuer; checkpoint capture carries the catalog publication frontier in the eager manifest, and cold installation restores it before any reservation. Deferred generation readmission rejects every identity at or above that frontier before it can enter the live catalog. No facade accepts a caller-chosen generation identity for this insertion. */
            !self.entries.contains_key(&generation.generation_id),
            "native index publication cannot replace a retained generation identity"
        );
        self.insert(generation);
    }

    pub(in crate::runtime::state::subsystems::indexing) fn insert(
        &mut self,
        generation: DerivedIndexGeneration,
    ) {
        assert_eq!(
            /* Proof: Native construction binds both branch axes to the admitted basis; checkpoint and envelope readmission reject mismatched transported axes before reaching this private catalog installation. */
            generation.source_branch_id,
            generation.applicability.branch_id,
            "generation source and applicability share one admitted branch"
        );
        self.generation_high_water = self.generation_high_water.max(generation.generation_id.0);
        if let Some(previous) = self.entries.remove(&generation.generation_id) {
            self.remove_bindings(&previous);
        }
        let mut branch = Some(generation.applicability.branch_id.clone());
        self.insert_with_branch(generation, &mut branch);
    }

    /// The borrowed cutover keeps any unused owned lookup key in its payload.
    pub(in crate::runtime::state::subsystems::indexing) fn publish_prepared(
        &mut self,
        generation: DerivedIndexGeneration,
        branch: &mut Option<BranchId>,
    ) {
        assert!(
            /* Proof: The same checked owner issuer reserves the batch's fresh identities before admission; the generation gate excludes every other publisher until this consuming installation. */
            !self.entries.contains_key(&generation.generation_id),
            "a prepared generation identity is fresh"
        );
        self.generation_high_water = self.generation_high_water.max(generation.generation_id.0);
        self.insert_with_branch(generation, branch);
    }

    fn insert_with_branch(
        &mut self,
        generation: DerivedIndexGeneration,
        branch: &mut Option<BranchId>,
    ) {
        let branches = self.scopes.entry(generation.index_id).or_default();
        branches.entry(None).or_default().insert(&generation);
        if let Some(scope) = branches.get_mut(branch) {
            scope.insert(&generation);
        } else {
            let key = branch
                .take()
                .expect("admission retains the exact branch key until installation");
            branches.entry(Some(key)).or_default().insert(&generation);
        }
        self.commits
            .entry(generation.source_commit_id)
            .or_default()
            .insert(generation.generation_id);
        self.versions
            .entry(generation.applicability.version_id)
            .or_default()
            .insert(generation.generation_id);
        self.entries
            .insert(generation.generation_id, Arc::new(generation));
    }

    pub(super) fn remove_bindings(&mut self, generation: &DerivedIndexGeneration) {
        if let Some(branches) = self.scopes.get_mut(&generation.index_id) {
            for branch in [None, Some(generation.applicability.branch_id.clone())] {
                if let Some(scope) = branches.get_mut(&branch) {
                    scope.remove(generation);
                    if scope.is_empty() {
                        branches.remove(&branch);
                    }
                }
            }
            if branches.is_empty() {
                self.scopes.remove(&generation.index_id);
            }
        }
        remove_member(
            &mut self.commits,
            &generation.source_commit_id,
            generation.generation_id,
        );
        remove_member(
            &mut self.versions,
            &generation.applicability.version_id,
            generation.generation_id,
        );
    }
}
