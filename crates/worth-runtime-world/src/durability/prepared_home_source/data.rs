//! World validates original DATA before Native preflight or Home descriptor effects.
use super::*;
use crate::durability::checkpoint::WorldCheckpointCoverage;
use crate::durability::{WorldCheckpointReadSource, WorldDurableHomeBinding};
use worth_relational::facade::durability::RelationalWorldSourceCatalog;

/// Immutable source DATA grants no service, product head, receipt or owner effect.
#[derive(Debug)]
pub struct PreparedWorldHomeData {
    state: Box<PreparedWorldHomeDataState>,
}
#[derive(Debug)]
struct PreparedWorldHomeDataState {
    expected: WorldDurableHomeBinding,
    recovered: WorldCheckpointImage,
    coverage: WorldCheckpointCoverage,
    _source: Box<dyn WorldCheckpointReadSource>,
    catalog: Arc<RelationalWorldSourceCatalog>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl PreparedWorldHomeData {
    pub fn from_home_source(
        expected: WorldDurableHomeBinding,
        source: Box<dyn WorldCheckpointReadSource>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut selected_native = None;
        let (recovered, coverage) = WorldCheckpointImage::read_protected_source(
            source.as_ref(),
            &expected,
            request,
            Some(&mut selected_native),
        )?;
        if recovered.branches.len() > 1 {
            return Err(Denial::OccurrenceReadmissionRequired);
        }
        for origin in recovered.branch_origins() {
            let crate::durability::WorldRecord::Branch {
                program_revision, ..
            } = &origin.record().record
            else {
                unreachable!("admitted branch origin")
            };
            if *program_revision != expected.program_revision() {
                return Err(Denial::ProgramRevisionMismatch);
            }
        }
        if recovered
            .opens
            .values()
            .any(|open| open.program_revision != expected.program_revision())
        {
            return Err(Denial::ProgramRevisionMismatch);
        }
        let catalog = native_catalog::prepare(&recovered, request, selected_native)?;
        let memory = request
            .reserve_retained_memory(size_of::<PreparedWorldHomeDataState>() as u64)
            .map_err(|cause| {
                Denial::Medium(super::super::WorldDurableMediumDenial::Request(cause))
            })?;
        Ok(Self {
            state: Box::new(PreparedWorldHomeDataState {
                expected,
                recovered,
                coverage,
                _source: source,
                catalog,
                _memory: memory,
            }),
        })
    }
    pub fn native_selection(&self) -> Option<Arc<RelationalWorldSourceCatalog>> {
        self.state
            .recovered
            .first_branch()
            .map(|_| self.state.catalog.clone())
    }
    pub(super) fn expected(&self) -> &WorldDurableHomeBinding {
        &self.state.expected
    }
    pub(super) fn recovered(&self) -> &WorldCheckpointImage {
        &self.state.recovered
    }
    pub(super) fn coverage(&self) -> &WorldCheckpointCoverage {
        &self.state.coverage
    }
}
