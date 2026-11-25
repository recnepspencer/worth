//! Independent cold oracle input never publishes serving residency.
use super::*;
use crate::storage::substrate::maintenance::ReconstructionLane;

pub struct ReconstructedRegionHead {
    region: Arc<ResidentRegion>,
    _custody: Arc<RelationalRegionResidency>,
}
impl ReconstructedRegionHead {
    pub fn view(&self) -> super::super::RelationalRegionHeadView<'_> {
        super::super::RelationalRegionHeadView::new(&self.region)
    }
}
impl RelationalRegionResidency {
    pub fn reconstruct_head(
        self: &Arc<Self>,
        _lane: &ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: Option<&super::super::RelationalRegionKindAuthority>,
    ) -> Result<ReconstructedRegionHead, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        self.check_damage()?;
        if let Some(authority) = authority {
            authority
                .require_cell(self)
                .map_err(|cause| self.failure(cause))?;
        }
        let resident = {
            let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            match &**state {
                ResidencyState::Head { region } | ResidencyState::Full(region) => {
                    Some(region.to_arc())
                }
                _ => None,
            }
        };
        let region = match resident {
            Some(region) => region,
            None => {
                let cold = self
                    .backing
                    .as_ref()
                    .ok_or_else(|| self.failure(Cause::SourceUnavailable))?;
                // Reuse only this original cell's authenticated input, without
                // publishing serving residency or trusting derived indexes.
                let source = {
                    let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
                    state
                        .physical
                        .as_ref()
                        .and_then(|physical| physical.source())
                        .cloned()
                        .ok_or_else(|| self.failure(Cause::SourceUnavailable))?
                };
                if self.working_set.get().is_some() {
                    // Bound cells never cache uncharged reconstruction reads in
                    // `raw_head`; subsequent `demand_head` must charge properly.
                    cold.read_head(&source, execution, 1 << 30)?
                } else {
                    cell_load_cache::load(&self.raw_head, self.partition_id, || {
                        cold.read_head(&source, execution, 1 << 30)
                    })?
                    .into_arc()
                }
            }
        };
        Ok(ReconstructedRegionHead {
            region,
            _custody: self.clone(),
        })
    }
}
