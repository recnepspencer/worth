//! Admit Runtime observations for explicit index reconstruction.
use super::super::live_access::IndexAuthority;
use super::{reconstruction_projection, recovery_catalog, request_payment};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::indexes::data::*;
use crate::storage::substrate::maintenance::ReconstructionLane;
use worth_execution::ExecutionRequest;

impl IndexAuthority<'_> {
    /// Explicit bounded recovery against an admitted immutable root, including a fork.
    pub fn reconstruct_for_basis(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        self.reconstruct_for_basis_observed(execution, request, basis, budget, None)
    }

    pub(crate) fn reconstruct_for_basis_observed(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        budget: DerivedIndexMaintenanceBudget,
        charge_rendezvous: Option<worth_relational_indexes::internal_port::indexes::authority::maintenance::charge_rendezvous::ChargeRendezvous<'_>>,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        ReconstructionLane::enter_reconstruction(|lane| {
            reconstruction_projection::reconstruct_for_basis(
                &self.maintenance(),
                lane,
                execution,
                &request,
                basis,
                budget,
                charge_rendezvous,
                |observation| self.runtime.read_truth().project_observation(observation),
            )
        })
    }

    /// Explicit bounded reconstruction for a committed version beyond the live branch head.
    pub fn reconstruct_for_commit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        ReconstructionLane::enter_reconstruction(|lane| {
            reconstruction_projection::reconstruct_for_commit(
                &self.maintenance(),
                &self.runtime.history,
                lane,
                execution,
                &request,
                budget,
                |observation| self.runtime.read_truth().project_observation(observation),
                |commit, branch, version| {
                    self.runtime
                        .read_truth()
                        .try_project_retained_commit(commit, branch, version)
                },
            )
        })
    }

    pub(crate) fn reconstruct_installed_for_basis(
        &self,
        lane: &ReconstructionLane<'_>,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            maximum_cold_record_slots: None,
            maximum_derived_rows: None,
        };
        let (prepared, counts) = request_payment::run(execution, budget, |work| {
            recovery_catalog::missing_build_request(
                &self.runtime.indexes,
                lane,
                basis,
                work,
                |observation| self.runtime.read_truth().project_observation(observation),
            )
        })?;
        let (request, funding) = prepared.into_funded_parts();
        if request.index_ids.is_empty() {
            return Ok(DerivedIndexMaintenanceOutcome {
                generations: Vec::new(),
                work: counts,
            });
        }
        let result = self.reconstruct_for_basis(execution, request, basis, budget);
        drop(funding);
        result
    }
}
