use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, PartitionId, RelationId};

use super::ObservedWorkflowAssessmentEvidence;
use super::{adjacency_with_kind, denial, exact, exact_u64, text};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    instance::{
        decode_transition_outcome, encode_transition_outcome, SettledWorkflowTransition,
        WorkflowTransitionLocator,
    },
    schema::WorthQueryWorkflowLayout,
};

mod dependency;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use dependency::decode_field_revision_fact;
mod proposal;
mod retained_evidence;
mod value;
pub(in crate::domain_computation::primary_graph::application_attempt) use dependency::observe_evidence_dependencies;
pub(in crate::domain_computation::primary_graph::application_attempt) use proposal::observe_retained_workflow_proposal_identity;
pub(in crate::domain_computation::primary_graph::application_attempt) use retained_evidence::observe_retained_assessment_evidence;
use value::{observed_bool, observed_text, observed_u64, optional_identity};

pub(super) fn observe_settled_transition(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<SettledWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let kind = layout.transition.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: transition,
        kind,
    });
    exact(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.protocol_version,
        AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        facts,
    )?;
    let occurrence = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.occurrence,
        facts,
    )?;
    let outcome = decode_transition_outcome(exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.outcome,
        facts,
    )?)
    .ok_or_else(|| denial("workflow transition outcome is unsupported"))?;
    let operation_receipt_identity = optional_identity(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.operation_receipt_identity,
        facts,
    )?;
    let nodes = adjacency_with_kind(
        runtime,
        snapshot,
        layout.transition_node_relation,
        transition,
        2,
        "workflow transition node relation is unavailable",
        facts,
    )?;
    let [node] = nodes.as_slice() else {
        return Err(denial("workflow transition node binding is not singular"));
    };
    Ok(SettledWorkflowTransition::new(
        *node,
        occurrence,
        outcome,
        operation_receipt_identity,
    ))
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_retained_transition(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    locator: WorkflowTransitionLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<SettledWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let observed = observe_settled_transition(runtime, snapshot, layout, locator.entity(), facts)?;
    if observed != locator.settlement() {
        return Err(denial("retained workflow transition locator changed"));
    }
    Ok(observed)
}

pub(super) fn observe_assessment_evidence(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<ObservedWorkflowAssessmentEvidence>, WorthQueryApplicationAttemptDenial> {
    let evidence = adjacency_with_kind(
        runtime,
        snapshot,
        layout.transition_assessment_evidence_relation,
        transition,
        2,
        "workflow assessment evidence relation is unavailable",
        facts,
    )?;
    let ([] | [_]) = evidence.as_slice() else {
        return Err(denial(
            "workflow transition has duplicate assessment evidence",
        ));
    };
    let Some(evidence) = evidence.first().copied() else {
        return Ok(None);
    };
    let kind = layout.assessment_evidence.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: evidence,
        kind,
    });
    exact(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.protocol_version,
        AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        facts,
    )?;
    let identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.identity,
        facts,
    )?;
    for locator in [
        &layout.assessment_evidence.producer,
        &layout.assessment_evidence.family,
    ] {
        observed_text(runtime, snapshot, evidence, kind, locator, facts)?;
    }
    let source_identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.source_identity,
        facts,
    )?;
    let publication_identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.publication_identity,
        facts,
    )?;
    let output_content_identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.output_content_identity,
        facts,
    )?;
    observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.proposal_identity,
        facts,
    )?;
    for locator in [
        &layout.assessment_evidence.subject_partition,
        &layout.assessment_evidence.subject_slot,
        &layout.assessment_evidence.subject_generation,
    ] {
        observed_u64(runtime, snapshot, evidence, kind, locator, facts)?;
    }
    Ok(Some(ObservedWorkflowAssessmentEvidence {
        entity: evidence,
        identity,
        query: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.query,
            facts,
        )?,
        parameter_type: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.parameter_type,
            facts,
        )?,
        result_type: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.result_type,
            facts,
        )?,
        binding: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.binding,
            facts,
        )?,
        passing: observed_bool(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.passing,
            facts,
        )?,
        coverage_identity: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.coverage_identity,
            facts,
        )?,
        source_identity,
        publication_identity,
        output_content_identity,
    }))
}

pub(in crate::domain_computation::primary_graph) fn recover_settled_live_membership(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    expected_identity: &str,
    expected_occurrence: u64,
) -> Result<(RelationId, Vec<WorthQueryApplicationObservedFact>), WorthQueryApplicationAttemptDenial>
{
    let kind = layout.transition.entity_kind;
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: transition,
        kind,
    }];
    exact(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.identity,
        text(expected_identity.to_owned()),
        &mut facts,
    )?;
    for (locator, expected) in [
        (
            &layout.transition.protocol_version,
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        (&layout.transition.occurrence, expected_occurrence),
        (
            &layout.transition.outcome,
            encode_transition_outcome(ApplicationWorkflowControlOutcome::Completed),
        ),
    ] {
        exact(
            runtime,
            snapshot,
            transition,
            kind,
            locator,
            AspectValue::UInt64(expected),
            &mut facts,
        )?;
    }
    let partition = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_partition,
        &mut facts,
    )?;
    let slot = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_slot,
        &mut facts,
    )?;
    let generation = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_generation,
        &mut facts,
    )?;
    let partition = u32::try_from(partition)
        .map_err(|_| denial("settled live-membership partition is invalid"))?;
    let generation = u32::try_from(generation)
        .map_err(|_| denial("settled live-membership generation is invalid"))?;
    Ok((
        RelationId::new(PartitionId::new(partition), slot, generation),
        facts,
    ))
}
