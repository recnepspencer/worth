//! Canonical source and output expectations retain their original backing.
use super::*;
pub(super) fn retain(
    candidate: &WorthQueryRetainedOutputCandidate,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    family: &'static str,
) -> Result<Option<FactPayload>, WorthQueryOutputDemandDenial> {
    let Some(inputs) = candidate
        .observed_source_facts
        .as_ref()
        .filter(|facts| !facts.is_empty())
    else {
        return Ok(None);
    };
    let Some(witness) = candidate
        .native_output_witness
        .as_ref()
        .and_then(|witness| witness.get())
    else {
        // Checkpoint admission retains the canonical output half in this payload.
        return Ok(matches!(candidate.source_identity, Some(crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(_))).then(|| inputs.facts().clone()));
    };
    let Some(output) = crate::domain_computation::primary_graph::output_reuse::owner_read(
        witness.prepare_fact_projection(),
        family,
    )?
    else {
        return Ok(None);
    };
    let storage = StorageControl::new(execution.allocation_policy(), Some(request))
        .with_execution(*execution);
    let retained = |cause: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial| candidate_support_denied(cause.into_attempt_denial(family));
    Ok(Some(
        inputs
            .facts()
            .append_completed(output.retain_payload(storage).map_err(retained)?, storage)
            .map_err(retained)?,
    ))
}
