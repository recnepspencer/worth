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
                let snapshot = {
                    let roots = self.inner.read_retired_roots();
                    roots
                        .get(&key)
                        .filter(|retired| retired.generation == generation && retired.retired)
                        .map(|retired| {
                            (
                                retired.root.clone(),
                                retired.historical_binding.clone(),
                                match branch {
                                    Some(branch) => retired
                                        .branch_bindings
                                        .get(branch)
                                        .cloned()
                                        .into_iter()
                                        .collect::<Vec<_>>(),
                                    None => retired
                                        .branch_bindings
                                        .values()
                                        .cloned()
                                        .collect::<Vec<_>>(),
                                },
                            )
                        })
                };
                let Some((root, historical, branches)) = snapshot else {
                    return false;
                };
                if root.commit_receipt().map(|receipt| receipt.version_id) != Some(version)
                    || root.schema_authority().schema_version() != schema
                {
                    return false;
                }
                let id = root.id();
                historical.has_root_obligation(id)
                    || branches
                        .iter()
                        .any(|binding| binding.has_root_obligation(id))
            })
    }
}
