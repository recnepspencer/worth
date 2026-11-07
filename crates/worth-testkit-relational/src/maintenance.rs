//! Shared equality and growth courts for the SDK's maintained structures.
use std::sync::Arc;
use worth_relational::facade::storage::maintenance::{
    ChangeWork, CommitChange, Maintained, ReconstructionLane,
};

#[derive(Debug)]
pub enum MaintenanceCourtFailure<E> {
    Apply(E),
    Reconstruction(E),
    Equality { phase: &'static str },
}

/// The caller builds genuine commit sequences and supplies an independently
/// reconstructed cold owner. The harness never constructs authority or fixtures.
pub struct MaintainedCourt<S> {
    previous: Arc<S>,
    phase: &'static str,
}
impl<S> MaintainedCourt<S> {
    pub fn new(phase: &'static str, previous: Arc<S>) -> Self {
        Self { previous, phase }
    }
    pub fn previous(&self) -> &Arc<S> {
        &self.previous
    }
    pub fn check_against_cold<E>(
        &mut self,
        updated: Arc<S>,
        rebuilt: Arc<S>,
        equivalent: impl FnOnce(&S, &S) -> bool,
    ) -> Result<Arc<S>, MaintenanceCourtFailure<E>> {
        if !equivalent(&updated, &rebuilt) {
            return Err(MaintenanceCourtFailure::Equality { phase: self.phase });
        }
        self.previous = updated.clone();
        Ok(updated)
    }

    pub fn advance<M: Maintained<State = S>>(
        &mut self,
        maintainer: &mut M,
        change: &CommitChange,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        equivalent: impl FnOnce(&S, &S) -> bool,
    ) -> Result<Arc<S>, MaintenanceCourtFailure<M::Error>> {
        let updated = ChangeWork::run(execution, |work| {
            worth_relational::facade::storage::maintenance::apply_change(
                maintainer,
                Some(&self.previous),
                change,
                work,
            )
        })
        .map_err(MaintenanceCourtFailure::Apply)?;
        let rebuilt = ReconstructionLane::enter_reconstruction(|lane| {
            lane.recover(execution, |lane, work| maintainer.cold(lane, work))
        })
        .map_err(MaintenanceCourtFailure::Reconstruction)?;
        self.check_against_cold(updated, rebuilt, equivalent)
    }
}

/// Growth only: a linear whole-state term approaches two; a lookup path has
/// generous headroom. Inputs are charged work from the existing request meter.
pub fn assert_change_bounded(phase: &str, axis: &str, base: u64, doubled: u64) {
    assert!(
        u128::from(doubled) * 2 < u128::from(base.max(1)) * 3,
        "{phase}: work grows with {axis}: base={base}, doubled={doubled}, ratio={:.3}",
        doubled as f64 / base.max(1) as f64
    );
}
