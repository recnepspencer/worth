//! Choose by validated producer occurrence, never by entity order or current pointers.
use super::super::history::operation_outputs::ObservedOperationOutputs;
use super::*;
use crate::domain_computation::primary_graph::workflow::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNodeKind,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccessionOutputMatch;
#[cfg(test)]
mod tests;

impl CompletedWorkflowSource {
    /// O(retained transitions) on the already reconstructed, exact completed basis.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn output_matching(
        &self,
        compiled: &CompiledWorkflowDefinition,
        matching: &ApplicationOperationBodySuccessionOutputMatch,
    ) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
        let occurrences = self.transitions.iter().filter_map(|transition| {
            transition.settlement.operation_receipt_identity()?;
            let CompiledWorkflowNodeKind::Operation {
                binding: Some(binding),
                ..
            } = compiled.node(transition.settlement.node())?.kind()
            else {
                return None;
            };
            Some(PerformedProducerOccurrence {
                occurrence: transition.settlement.occurrence(),
                binding,
                outputs: transition.operation_outputs.as_ref(),
            })
        });
        latest_output(occurrences, matching)
    }
}

/// Descriptive input to selection, not a workflow occurrence/command authority.
struct PerformedProducerOccurrence<'a> {
    occurrence: u64,
    binding: &'a str,
    outputs: Option<&'a ObservedOperationOutputs>,
}

fn latest_output<'a>(
    occurrences: impl Iterator<Item = PerformedProducerOccurrence<'a>>,
    matching: &ApplicationOperationBodySuccessionOutputMatch,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let unavailable = || {
        WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowPredecessorOutputUnavailable,
            "latest declared predecessor producer has no exact retained output",
        )
    };
    let latest = occurrences
        .filter(|occurrence| occurrence.binding == matching.producer_binding())
        .max_by_key(|occurrence| occurrence.occurrence)
        .ok_or_else(unavailable)?;
    // Select the occurrence first: never fall back when its context/role is missing.
    let outputs = latest.outputs.ok_or_else(unavailable)?;
    let role = matching.producer_role();
    let output = outputs.entries.get(&role.name).ok_or_else(unavailable)?;
    if outputs.binding != matching.producer_binding()
        || output.entity != role.entity
        || output.posture != role.posture
    {
        return Err(unavailable());
    }
    Ok(output.target)
}
