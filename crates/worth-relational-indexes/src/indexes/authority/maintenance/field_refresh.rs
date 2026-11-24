use super::work::{MaintenanceAdmissionRefusal, MaintenanceWork};
use super::PreparedRefresh;
use crate::contracts::indexes::data::maintenance::{
    DerivedIndexMaintenanceAdmissionStop, DerivedIndexMaintenanceBudget,
    DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::runtime::state::subsystems::indexing::IndexingSubsystem;

use worth_relational_branch::internal_port::indexes::data::maintenance_outcome::DerivedIndexMaintenanceOutcome;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexBuildRequest;

/// Maintains only selected field indexes on the original request's work and memory payer.
/// Reuses exact native generations, including a fork authoring root. Population reconstruction
/// is refused; the complete selected batch publishes atomically or not at all.
pub fn refresh_field_indexes_for_basis(
    indexes: &IndexingSubsystem,
    request: DerivedIndexBuildRequest,
    budget: DerivedIndexMaintenanceBudget,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut prepare_refresh: impl FnMut(
        &DerivedIndexBuildRequest,
        &mut MaintenanceWork<'_, '_, '_>,
    ) -> Result<PreparedRefresh, Denial>,
) -> Result<
    DerivedIndexMaintenanceOutcome,
    crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenial,
> {
    let (prepared, counts) = super::request_payment::run(execution, budget, |work| {
        work.restrict_to_field_indexes();
        let prepared = prepare_refresh(&request, work)?;
        prepared.publish_admitted(indexes, work)
    })?;
    Ok(DerivedIndexMaintenanceOutcome {
        generations: prepared.value,
        work: counts,
    })
}

/// Maintains selected field indexes through the original preparation payer.
/// Missing predecessor state refuses, independently of the cold-record budget.
/// The first caller refusal remains sticky and intact.
pub fn refresh_field_indexes_for_basis_admitted<Stop>(
    indexes: &IndexingSubsystem,
    request: DerivedIndexBuildRequest,
    budget: DerivedIndexMaintenanceBudget,
    mut prepare_refresh: impl FnMut(
        &DerivedIndexBuildRequest,
        &mut MaintenanceWork<'_, '_, '_>,
    ) -> Result<PreparedRefresh, Denial>,
    mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceAdmissionStop<Stop>> {
    let mut refused = None;
    let result = (|| {
        let mut relay = |work, bytes| {
            if refused.is_some() {
                return Err(MaintenanceAdmissionRefusal);
            }
            prepare(work, bytes).map_err(|stop| {
                refused = Some(stop);
                MaintenanceAdmissionRefusal
            })
        };
        let mut work = MaintenanceWork::admitted(budget, &mut relay);
        work.restrict_to_field_indexes();
        let prepared = prepare_refresh(&request, &mut work).map_err(|kind| work.deny(kind))?;
        // A caught internal native error cannot bypass a caller refusal.
        work.prepare(0, 0).map_err(|kind| work.deny(kind))?;
        Ok(DerivedIndexMaintenanceOutcome {
            generations: prepared
                .publish_admitted(indexes, &mut work)
                .map_err(|kind| work.deny(kind))?,
            work: work.counts(),
        })
    })();
    match refused {
        Some(stop) => Err(DerivedIndexMaintenanceAdmissionStop::Admission(stop)),
        None => result.map_err(DerivedIndexMaintenanceAdmissionStop::Native),
    }
}
