//! Consume complete captured metadata into its sealed ordinary manifest.
use super::*;

impl CapturedManifest {
    /// A native capture carries pages its destination holds (or, with no
    /// known membership, the prior's pages when metadata may carry) and takes
    /// its coverage from the paged index's inline tail.
    pub(crate) fn seal_native_commit_index(
        &mut self,
        current_members: Option<&std::collections::BTreeSet<RelationalDurableMemberLocator>>,
        carry_metadata: bool,
    ) -> Result<Vec<RelationalCheckpointMember>, DurabilityError> {
        use crate::durability::log::native_checkpoint_codec::commit_pages::CommitPageCarry;
        let carry = current_members.map_or(
            CommitPageCarry::Prior(carry_metadata),
            CommitPageCarry::Held,
        );
        let pages = self.seal_commit_index(carry)?;
        let tail = self.manifest.commits.tail.as_ref();
        self.manifest.coverage.up_to_commit = tail.map(|entry| entry.commit.clone());
        self.manifest.coverage.up_to_version = tail.map(|entry| entry.commit.version_id);
        Ok(pages)
    }

    /// Pages the captured rows into the manifest's commit index, carrying the
    /// prior index's unchanged prefix pages; returns the page members.
    pub(crate) fn seal_commit_index(
        &mut self,
        carry: crate::durability::log::native_checkpoint_codec::commit_pages::CommitPageCarry<'_>,
    ) -> Result<Vec<RelationalCheckpointMember>, DurabilityError> {
        let roots = self
            .manifest
            .roots
            .iter()
            .map(|root| root.commit_id)
            .collect();
        let (index, members) =
            crate::durability::log::native_checkpoint_codec::commit_pages::page_commit_rows(
                self.prior_commits.as_deref(),
                &self.commit_rows,
                &roots,
                carry,
            )?;
        self.manifest.commits = index;
        Ok(members)
    }

    pub(crate) fn seal_ordinary(self) -> CapturedManifest<RelationalCheckpointManifest> {
        CapturedManifest {
            manifest: self.manifest.seal_ordinary(),
            component_holds: self.component_holds,
            members: self.members,
            canonical_publication: self.canonical_publication,
            canonical_payloads: self.canonical_payloads,
            envelope_cells: self.envelope_cells,
            publications: self.publications,
            cold_source: self.cold_source,
            guarded_members: self.guarded_members,
            history_members: self.history_members,
            commit_rows: self.commit_rows,
            prior_commits: self.prior_commits,
            prior_canonical: self.prior_canonical,
            prior_source: self.prior_source,
        }
    }
}
