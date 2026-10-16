use super::work::{MaintenanceAdmissionRefusal, MaintenanceWork};
use super::IndexAuthority;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexMaintenanceAdmissionStop, DerivedIndexMaintenanceBudget,
    DerivedIndexMaintenanceOutcome,
};

impl IndexAuthority<'_> {
    /// Reconstructs only selected field indexes on the original request's work and memory payer.
    /// Reuses exact native generations, including a fork authoring root. Population reconstruction
    /// is refused; the complete selected batch publishes atomically or not at all.
    pub fn refresh_field_indexes_for_basis(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        before: Option<&crate::snapshots::data::SnapshotHandle>,
        budget: DerivedIndexMaintenanceBudget,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DerivedIndexMaintenanceOutcome, crate::indexes::data::DerivedIndexMaintenanceDenial>
    {
        let (prepared, counts) = super::request_payment::run(execution, budget, |work| {
            work.restrict_to_field_indexes();
            let prepared = self.prepare_basis_refresh(&request, basis, before, work, execution)?;
            prepared.publish_admitted(self.runtime, work)
        })?;
        Ok(DerivedIndexMaintenanceOutcome {
            generations: prepared.value,
            work: counts,
        })
    }

    /// Reconstructs selected field indexes through the native maintenance owner
    /// on the caller's preparation authority. Other index kinds are explicitly
    /// unavailable in this contract; the ordinary maintenance API supports them.
    /// A zero cold-record allowance requires an existing exact or authentic fork
    /// generation; it refuses before population reconstruction. A positive cold
    /// allowance explicitly admits bounded reconstruction on this same payer.
    /// The first caller refusal remains sticky and intact.
    pub fn refresh_field_indexes_for_basis_admitted<Stop>(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        budget: DerivedIndexMaintenanceBudget,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            if budget.maximum_cold_record_slots == 0 {
                work.restrict_to_field_indexes();
            }
            let prepared = self
                .prepare_basis_refresh(&request, basis, None, &mut work, execution)
                .map_err(|kind| work.deny(kind))?;
            // A caught internal native error cannot bypass a caller refusal.
            work.prepare(0, 0).map_err(|kind| work.deny(kind))?;
            Ok(DerivedIndexMaintenanceOutcome {
                generations: prepared
                    .publish_admitted(self.runtime, &mut work)
                    .map_err(|kind| work.deny(kind))?,
                work: work.counts(),
            })
        })();
        match refused {
            Some(stop) => Err(DerivedIndexMaintenanceAdmissionStop::Admission(stop)),
            None => result.map_err(DerivedIndexMaintenanceAdmissionStop::Native),
        }
    }
}
