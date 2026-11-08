//! Verify data at D while origin control remains at exact C.
use super::*;
pub(super) fn holds<S: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before: &crate::basis::WorthQueryProductBranchLease,
    pin: &Pin,
    candidate: &WorthQueryRetainedOutputCandidate,
    facts: &FactPayload,
    family: &'static str,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    application.runtime.primary_provider.graph.with_runtime_mut(execution, |runtime| {
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
            runtime, before.relational_basis(),
        ).map_err(|cause| WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            format!("{family}: {cause:?}"),
        ))?;
        let answer = (|| {
            if !origin_is_verified(application, runtime, &snapshot, candidate)? {
                return Ok(false);
            }
            for fact in facts.iter() {
                let equal = fact.remains_equal_in_image(runtime, execution, &snapshot, Some(pin))
                    .map_err(candidate_support_denied)?;
                if !equal {
                    return Ok(false);
                }
            }
            Ok::<_, WorthQueryOutputDemandDenial>(true)
        })();
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        answer
    })
}
