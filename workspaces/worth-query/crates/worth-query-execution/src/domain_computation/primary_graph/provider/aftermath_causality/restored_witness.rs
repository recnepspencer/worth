//! Validate a sealed reversal at the caller's exact current Product source.
use super::{canonical_record, WorthQueryPrimaryGraphProvider};
use crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality;
use crate::domain_computation::primary_graph::provider::CorrectionWitnessDenial;
use worth_execution::ExecutionRequest;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn validate_correction_witness(
        &self,
        execution: ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        sealed: &WorthQueryCommittedAftermathCausality,
    ) -> Result<bool, CorrectionWitnessDenial> {
        execution
            .consult()
            .map_err(CorrectionWitnessDenial::Request)?;
        self.graph.with_runtime_mut(&execution, |runtime| {
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                .map_err(CorrectionWitnessDenial::Basis)?;
            let result = canonical_record::matches_reverse(runtime, &snapshot,
                self.graph.layout.provider_aftermath_causality(), execution, sealed);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            result
        })
    }
}
