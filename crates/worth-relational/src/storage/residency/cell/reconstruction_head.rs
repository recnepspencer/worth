//! Independent cold oracle input never publishes serving residency.
use super::*;
use crate::storage::substrate::maintenance::ReconstructionLane;

pub(crate) struct ReconstructedRegionHead {
    region: Arc<ResidentRegion>,
    _custody: Arc<RelationalRegionResidency>,
}
impl ReconstructedRegionHead {
    pub(crate) fn view(&self) -> super::super::RelationalRegionHeadView<'_> {
        super::super::RelationalRegionHeadView::new(&self.region)
    }
}
impl RelationalRegionResidency {
    pub(crate) fn reconstruct_head(
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
                    Some(region.clone())
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
                cell_load_cache::load(&self.raw_head, self.partition_id, || {
                    region_load::load_head(cold, &source, execution, 1 << 30)
                })?
            }
        };
        Ok(ReconstructedRegionHead {
            region,
            _custody: self.clone(),
        })
    }
}
