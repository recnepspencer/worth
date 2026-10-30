//! Exact registered reader custody for one derived generation's canonical basis.

use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::schema::data::SchemaVersionId;

impl super::RelationalBranchRetentionOwner {
    /// Work follows physical roots and canonical aliases of this exact basis
    /// and, for a global generation, their actual branch bindings.
    pub(crate) fn index_basis_is_held(
        &self,
        version: VersionId,
        schema: SchemaVersionId,
        branch: Option<&BranchId>,
    ) -> bool {
        self.inner
            .retired_root_locators
            .snapshot_basis(version, schema)
            .into_iter()
            .any(|(key, generation)| {
                self.inner
                    .index_retention_roots_examined
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(retired) = self.inner.retired_roots.get(&key) else {
                    return false;
                };
                if retired.generation != generation || !retired.retired {
                    return false;
                }
                let root = &retired.root;
                if root
                    .canonical_envelope()
                    .map(|envelope| envelope.commit.version_id)
                    != Some(version)
                    || root.schema_authority().schema_version() != schema
                {
                    return false;
                }
                let id = root.id();
                retired.historical_binding.has_root_obligation(id)
                    || match branch {
                        Some(branch) => retired
                            .branch_bindings
                            .get(branch)
                            .is_some_and(|binding| binding.has_root_obligation(id)),
                        None => retired
                            .branch_bindings
                            .values()
                            .any(|binding| binding.has_root_obligation(id)),
                    }
            })
    }
}
