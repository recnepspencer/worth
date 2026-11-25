//! Explicit region demand and raw readmission under the original cell gate.
use super::*;
use crate::storage::residency::working_set::ResidencyLevelKind;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCauseOwnerPort as _;
pub mod readmission_count;
impl RelationalRegionResidency {
    pub(super) fn demand(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        demand: ResidencyDemand,
        limit: u64,
        authority: Option<&super::super::RelationalRegionKindAuthority>,
    ) -> Result<super::SharedResidentRegion, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        if let Some(authority) = authority {
            // A foreign root's proof is a refused demand, not damage to this
            // cell's independently authenticated members.
            authority
                .require_cell(self)
                .map_err(|cause| self.failure(cause))?;
        }
        // M4 §4.3: working-set bytes are reserved before this cell's gate.
        let mut admission = self.admit_working_set(demand);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let loaded = (|| {
            self.check_damage()?;
            if let ResidencyState::Cold = &**state {
                self.require_capacity(admission.as_ref())?;
                let region = self.raw_head(
                    admission.as_mut(),
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
                self.require_capacity(admission.as_ref())?;
                let region = self.raw_full(
                    admission.as_mut(),
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
        match readmission_count::acquire(loaded, &self.leases, self.partition_id) {
            Ok(region) => {
                self.mark_referenced();
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
            Err(Failure::Content(cause)) => Err(Failure::Content(cause)),
        }
    }

    // Raw readmission authenticates record content independently of derived
    // buckets. Keeping it separate breaks cross-partition adjacency cycles:
    // bucket verification never waits for another region's serving admission.
    fn raw_head(
        &self,
        admission: Option<&mut super::working_set_charge::WorkingSetAdmission>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
        source: &Arc<dyn RelationalDurableColdSource>,
    ) -> Result<super::SharedResidentRegion, Failure> {
        cell_load_cache::load(&self.raw_head, self.partition_id, || {
            let cold = self
                .backing
                .as_ref()
                .ok_or_else(|| self.failure(Cause::SourceUnavailable))?;
            let region = cold.read_head(source, execution, limit)?;
            self.charge_loaded(
                region,
                admission,
                ResidencyLevelKind::Head,
                cold.head.length,
            )
        })
    }

    fn raw_full(
        &self,
        admission: Option<&mut super::working_set_charge::WorkingSetAdmission>,
        head: &ResidentRegion,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
        source: Option<&Arc<dyn RelationalDurableColdSource>>,
    ) -> Result<super::SharedResidentRegion, Failure> {
        cell_load_cache::load(&self.raw_full, self.partition_id, || match &self.backing {
            Some(cold) => {
                let region = cold.read_full(
                    source.ok_or_else(|| self.failure(Cause::SourceUnavailable))?,
                    head,
                    execution,
                    limit,
                )?;
                let encoded = cold.head.length.saturating_add(cold.history.length);
                self.charge_loaded(region, admission, ResidencyLevelKind::Full, encoded)
            }
            None => self
                .head_readmission
                .as_ref()
                .ok_or_else(|| self.failure(Cause::SourceUnavailable))?
                .load_full(head, execution, limit),
        })
    }
}
