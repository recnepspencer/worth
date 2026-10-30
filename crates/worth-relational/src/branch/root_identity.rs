use super::{RelationalBranchRoot, RelationalBranchRootCaptureDenial};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchRootIdentityIssuer {
    counters: Arc<RelationalBranchRootIdentityCounters>,
}

#[derive(Debug)]
struct RelationalBranchRootIdentityCounters {
    next_root_id: AtomicU64,
    next_schema_authority_id: AtomicU64,
    next_region_id: AtomicU64,
    next_reachability_id: AtomicU64,
}

impl Default for RelationalBranchRootIdentityIssuer {
    fn default() -> Self {
        Self {
            counters: Arc::new(RelationalBranchRootIdentityCounters {
                next_root_id: AtomicU64::new(1),
                next_schema_authority_id: AtomicU64::new(1),
                next_region_id: AtomicU64::new(1),
                next_reachability_id: AtomicU64::new(1),
            }),
        }
    }
}

impl RelationalBranchRootIdentityIssuer {
    pub(super) fn next_reachability_id(&self) -> u64 {
        self.counters.next_reachability_id.load(Ordering::Relaxed)
    }

    pub(crate) fn observe_root(&self, root: &RelationalBranchRoot) {
        self.counters
            .next_root_id
            .fetch_max(root.id.saturating_add(1), Ordering::Relaxed);
        self.counters.next_schema_authority_id.fetch_max(
            root.schema_authority().allocation_id().saturating_add(1),
            Ordering::Relaxed,
        );
        let next_region_id = root
            .storage_regions()
            .map(|region| region.region_id.saturating_add(1))
            .max()
            .unwrap_or_else(|| self.counters.next_region_id.load(Ordering::Relaxed));
        self.counters
            .next_region_id
            .fetch_max(next_region_id, Ordering::Relaxed);
        let next_reachability_id = root
            .regions
            .allocation_observations()
            .into_iter()
            .map(|node| node.node_id.saturating_add(1))
            .max()
            .unwrap_or_else(|| self.counters.next_reachability_id.load(Ordering::Relaxed));
        self.counters
            .next_reachability_id
            .fetch_max(next_reachability_id, Ordering::Relaxed);
    }

    pub(crate) fn validate_capture_capacity(
        &self,
        touched_regions: usize,
    ) -> Result<(), RelationalBranchRootCaptureDenial> {
        self.counters
            .next_root_id
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(RelationalBranchRootCaptureDenial::RootIdentityExhausted)?;
        self.counters
            .next_schema_authority_id
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(RelationalBranchRootCaptureDenial::SchemaAuthorityIdentityExhausted)?;
        self.counters
            .next_region_id
            .load(Ordering::Relaxed)
            .checked_add(touched_regions as u64)
            .ok_or(RelationalBranchRootCaptureDenial::RegionIdentityExhausted)?;
        let maximum_path_nodes = (touched_regions as u64)
            .checked_mul(33)
            .and_then(|nodes| nodes.checked_add(1))
            .ok_or(RelationalBranchRootCaptureDenial::ReachabilityIdentityExhausted)?;
        self.counters
            .next_reachability_id
            .load(Ordering::Relaxed)
            .checked_add(maximum_path_nodes)
            .ok_or(RelationalBranchRootCaptureDenial::ReachabilityIdentityExhausted)?;
        Ok(())
    }

    pub(super) fn issue_root_id(&self) -> Result<u64, RelationalBranchRootCaptureDenial> {
        issue(
            &self.counters.next_root_id,
            RelationalBranchRootCaptureDenial::RootIdentityExhausted,
        )
    }

    pub(super) fn issue_schema_authority_id(
        &self,
    ) -> Result<u64, RelationalBranchRootCaptureDenial> {
        issue(
            &self.counters.next_schema_authority_id,
            RelationalBranchRootCaptureDenial::SchemaAuthorityIdentityExhausted,
        )
    }

    pub(super) fn issue_region_id(&self) -> Result<u64, RelationalBranchRootCaptureDenial> {
        issue(
            &self.counters.next_region_id,
            RelationalBranchRootCaptureDenial::RegionIdentityExhausted,
        )
    }

    pub(crate) fn issue_reachability_id(&self) -> Result<u64, RelationalBranchRootCaptureDenial> {
        issue(
            &self.counters.next_reachability_id,
            RelationalBranchRootCaptureDenial::ReachabilityIdentityExhausted,
        )
    }

    /// Capture an independent allocator frontier for a detached runtime fork.
    pub(crate) fn detached_owner_snapshot(&self) -> Self {
        Self {
            counters: Arc::new(RelationalBranchRootIdentityCounters {
                next_root_id: AtomicU64::new(self.counters.next_root_id.load(Ordering::Relaxed)),
                next_schema_authority_id: AtomicU64::new(
                    self.counters
                        .next_schema_authority_id
                        .load(Ordering::Relaxed),
                ),
                next_region_id: AtomicU64::new(
                    self.counters.next_region_id.load(Ordering::Relaxed),
                ),
                next_reachability_id: AtomicU64::new(
                    self.counters.next_reachability_id.load(Ordering::Relaxed),
                ),
            }),
        }
    }
}

fn issue(
    counter: &AtomicU64,
    denial: RelationalBranchRootCaptureDenial,
) -> Result<u64, RelationalBranchRootCaptureDenial> {
    {
        let update_counter = counter;
        let mut current = update_counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = current.checked_add(1) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .map_err(|_| denial)
}
