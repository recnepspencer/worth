//! Inspect one root's original lanes outside the primary inventory gate.
use super::*;
impl RelationalBranchRetentionOwner {
    pub(crate) fn retained_index_roots(&self) -> super::super::RetainedIndexRoots {
        use std::ops::Bound::{Excluded, Unbounded};
        let mut retained = super::super::RetainedIndexRoots::default();
        let mut after = None;
        loop {
            let next = {
                let roots = self.inner.read_retired_roots();
                let next = match after {
                    None => roots.first_key_value(),
                    Some(key) => roots.range((Excluded(key), Unbounded)).next(),
                };
                next.map(|(&key, entry)| {
                    (
                        key,
                        entry.generation,
                        entry.root.clone(),
                        entry.retired,
                        entry.branch_bindings.clone(),
                        entry.historical_binding.clone(),
                    )
                })
            };
            let Some((key, generation, root, retired, bindings, historical)) = next else {
                break;
            };
            after = Some(key);
            self.inner
                .index_retention_roots_examined
                .fetch_add(1, Ordering::Relaxed);
            if !retired {
                continue;
            }
            let Some(receipt) = root.commit_receipt() else {
                continue;
            };
            let version = receipt.version_id;
            let schema = root.schema_authority().schema_version();
            let id = root.id();
            let liveness = bindings
                .iter()
                .map(|(branch, binding)| (branch, binding, binding.has_root_obligation(id)))
                .collect::<Vec<_>>();
            let historical_live = historical.has_root_obligation(id);
            let mut roots = self.inner.write_retired_roots();
            let Some(entry) = roots.get_mut(&key) else {
                continue;
            };
            if entry.generation != generation || !entry.retired || !Arc::ptr_eq(&entry.root, &root)
            {
                continue;
            }
            for (branch, observed, live) in &liveness {
                if !live
                    && entry
                        .branch_bindings
                        .get(*branch)
                        .is_some_and(|current| current.same_lane(observed))
                {
                    // `bindings` still holds this exact original lane, so no
                    // lane payload is destroyed under the primary write gate.
                    entry.branch_bindings.remove(*branch);
                }
            }
            for branch in entry.branch_bindings.keys() {
                retained.retired.insert((branch.clone(), version, schema));
            }
            if historical_live && entry.historical_binding.same_lane(&historical) {
                retained.historical_versions.insert((version, schema));
            }
            drop(roots);
            // Removed lane and immutable root payloads are released only here.
        }
        retained
    }
}
