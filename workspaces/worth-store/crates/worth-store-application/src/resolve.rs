//! Home-owned routing to the native journal authority.
impl crate::ReadmittedApplicationHome {
    pub fn resolve(
        &self,
        position: worth_relational::facade::publication::PatchStreamPosition,
        digest: worth_relational::facade::durability::RelationalCanonicalCommitArtifactDigest,
    ) -> Result<
        worth_store_relational::RelationalStoreCommitFate,
        worth_store_relational::RelationalStoreJournalRefusal,
    > {
        self.relational.resolve(position, digest)
    }

    pub fn resolve_branch_transition(
        &self,
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
        digest: worth_relational::facade::durability::RelationalBranchTransitionArtifactDigest,
    ) -> Result<
        worth_store_relational::RelationalStoreTransitionFate,
        worth_store_relational::RelationalStoreTransitionJournalRefusal,
    > {
        self.relational.resolve_branch_transition(sequence, digest)
    }
}
