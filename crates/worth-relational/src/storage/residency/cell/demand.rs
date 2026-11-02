//! Explicit region demand and raw readmission under the original cell gate.
use super::*;
impl RelationalRegionResidency {
    pub(super) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        demand: ResidencyDemand,
        limit: u64,
        authority: Option<&super::super::RelationalRegionKindAuthority>,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        if let Some(authority) = authority {
            // A foreign root's proof is a refused demand, not damage to this
            // cell's independently authenticated members.
            authority
                .require_cell(self)
                .map_err(|cause| self.failure(cause))?;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let loaded = (|| {
            self.check_damage()?;
            if let ResidencyState::Cold = &**state {
                let region = self.raw_head(
                    execution,
                    limit,
                    state
                        .physical
                        .as_ref()
                        .and_then(|physical| physical.source())
                        .ok_or_else(|| self.failure(Cause::SourceUnavailable))?,
                )?;
                **state = ResidencyState::Head { region };
            }
            if matches!(demand, ResidencyDemand::Full)
                && let ResidencyState::Head { region } = &**state
            {
                let region = self.raw_full(
                    region,
                    execution,
                    limit,
                    state
                        .physical
                        .as_ref()
                        .and_then(|physical| physical.source()),
                )?;
                **state = ResidencyState::Full(region);
            }
            match &**state {
                ResidencyState::Head { region, .. } | ResidencyState::Full(region) => {
                    Ok(region.clone())
                }
                _ => Err(Failure::StorageUnavailable(
                    self.failure(Cause::SourceUnavailable),
                )),
            }
        })();
        match loaded {
            Ok(region) => {
                {
                    let update_counter = &self.leases;
                    let mut count = update_counter.load(Ordering::Acquire);
                    loop {
                        let Some(updated_value) = count.checked_add(1) else {
                            break Err(count);
                        };
                        match update_counter.compare_exchange_weak(
                            count,
                            updated_value,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        ) {
                            Ok(previous) => break Ok(previous),
                            Err(reloaded_value) => count = reloaded_value,
                        }
                    }
                }
                .map_err(|_| self.failure(Cause::MemoryExhausted))?;
                Ok(region)
            }
            Err(Failure::StorageUnavailable(failure)) => {
                if failure.cause.is_proven_damage()
                    && failure.subject == RelationalStorageSubject::Region(self.partition_id)
                {
                    let slot_counts = match &**state {
                        ResidencyState::Head { region } | ResidencyState::Full(region) => (
                            region.partition.entity_arena.slot_count(),
                            region.partition.relation_arena.slot_count(),
                        ),
                        ResidencyState::Cold => self
                            .backing
                            .as_ref()
                            .map_or((0, 0), |cold| (cold.entity_slots, cold.relation_slots)),
                        ResidencyState::Damaged { slot_counts } => *slot_counts,
                    };
                    let _ = self.damage.set(failure);
                    **state = ResidencyState::Damaged { slot_counts };
                }
                Err(Failure::StorageUnavailable(failure))
            }
            Err(Failure::Request(cause)) => Err(Failure::Request(cause)),
        }
    }

    // Raw readmission authenticates record content independently of derived
    // buckets. Keeping it separate breaks cross-partition adjacency cycles:
    // bucket verification never waits for another region's serving admission.
    fn raw_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
        source: &Arc<dyn RelationalDurableColdSource>,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        cell_load_cache::load(&self.raw_head, self.partition_id, || {
            let cold = self
                .backing
                .as_ref()
                .ok_or_else(|| self.failure(Cause::SourceUnavailable))?;
            region_load::load_head(cold, source, execution, limit)
        })
    }

    fn raw_full(
        &self,
        head: &Arc<ResidentRegion>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
        source: Option<&Arc<dyn RelationalDurableColdSource>>,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        cell_load_cache::load(&self.raw_full, self.partition_id, || match &self.backing {
            Some(cold) => region_load::load_full(
                cold,
                source.ok_or_else(|| self.failure(Cause::SourceUnavailable))?,
                head,
                execution,
                limit,
            ),
            None => self
                .head_readmission
                .as_ref()
                .ok_or_else(|| self.failure(Cause::SourceUnavailable))?
                .load_full(head, execution, limit),
        })
    }
}
