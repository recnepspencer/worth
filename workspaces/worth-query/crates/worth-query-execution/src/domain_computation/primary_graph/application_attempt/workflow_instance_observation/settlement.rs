mod membership;
pub(in crate::domain_computation::primary_graph) use membership::recover_settled_live_membership;
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

mod approval;
mod dependency;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use dependency::decode_field_revision_fact;
mod proposal;
mod retained_evidence;
mod value;
pub(in crate::domain_computation::primary_graph::application_attempt) use approval::observe_retained_approval_binding;
pub(in crate::domain_computation::primary_graph::application_attempt) use dependency::observe_evidence_dependencies;
pub(in crate::domain_computation::primary_graph::application_attempt) use proposal::observe_retained_workflow_proposal_identity;
pub(in crate::domain_computation::primary_graph::application_attempt) use retained_evidence::observe_retained_assessment_evidence;
use value::{
    observed_bool, observed_optional_text, observed_text, observed_u64, optional_identity,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum TransitionReadPurpose {
    Command,
    ReadOnly,
}
pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_settled_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<SettledWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    observe_transition(
        execution,
        runtime,
        snapshot,
        layout,
        transition,
        facts,
        TransitionReadPurpose::Command,
    )
}
pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_transition_state(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<SettledWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    observe_transition(
        execution,
        runtime,
        snapshot,
        layout,
        transition,
        facts,
        TransitionReadPurpose::ReadOnly,
    )
}
fn observe_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
    purpose: TransitionReadPurpose,
) -> Result<SettledWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let kind = layout.transition.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: transition,
        kind,
    });
    exact(execution,
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
        execution,
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.occurrence,
        facts,
    )?;
    let outcome = decode_transition_outcome(exact_u64(
        execution,
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.outcome,
        facts,
    )?)
    .ok_or_else(|| denial("workflow transition outcome is unsupported"))?;
    let operation_receipt_identity = optional_identity(
        execution,
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.operation_receipt_identity,
        facts,
    )?;
    if purpose == TransitionReadPurpose::Command {
        let advancer = layout
            .transition
            .advancer
            .observe(runtime, execution, snapshot, transition, kind, facts)?;
        let executor = layout
            .transition
            .executor
            .observe(runtime, execution, snapshot, transition, kind, facts)?;
        let applicable = crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability;
        if applicable(advancer, true).is_err()
            || applicable(executor, operation_receipt_identity.is_some()).is_err()
        {
            return Err(denial("current transition actor applicability is corrupt"));
        }
    }
    let nodes = adjacency_with_kind(
        execution,
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    locator: WorkflowTransitionLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<SettledWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let observed = observe_settled_transition(
        execution,
        runtime,
        snapshot,
        layout,
        locator.entity(),
        facts,
    )?;
    if observed != locator.settlement() {
        return Err(denial("retained workflow transition locator changed"));
    }
    Ok(observed)
}

pub(super) fn observe_assessment_evidence(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<ObservedWorkflowAssessmentEvidence>, WorthQueryApplicationAttemptDenial> {
    let evidence = adjacency_with_kind(
        execution,
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
    exact(execution,
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
        execution,
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
        observed_text(execution, runtime, snapshot, evidence, kind, locator, facts)?;
    }
    let source_identity = observed_text(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.source_identity,
        facts,
    )?;
    let publication_identity = observed_text(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.publication_identity,
        facts,
    )?;
    let output_content_identity = observed_text(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.output_content_identity,
        facts,
    )?;
    let program_revision = observed_text(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.program_revision,
        facts,
    )?;
    let proposal_identity = observed_text(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.proposal_identity,
        facts,
    )?;
    let partition = observed_u64(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.subject_partition,
        facts,
    )?;
    let slot = observed_u64(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.subject_slot,
        facts,
    )?;
    let generation = observed_u64(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.subject_generation,
        facts,
    )?;
    let retained_bytes = observed_u64(
        execution,
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.retained_bytes,
        facts,
    )?;
    let partition = u32::try_from(partition)
        .map_err(|_| denial("assessment evidence subject partition is malformed"))?;
    let generation = u32::try_from(generation)
        .map_err(|_| denial("assessment evidence subject generation is malformed"))?;
    Ok(Some(ObservedWorkflowAssessmentEvidence {
        entity: evidence,
        identity,
        retained_bytes,
        query: observed_text(
            execution,
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.query,
            facts,
        )?,
        parameter_type: observed_text(
            execution,
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.parameter_type,
            facts,
        )?,
        result_type: observed_text(
            execution,
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.result_type,
            facts,
        )?,
        binding: observed_text(
            execution,
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.binding,
            facts,
        )?,
        passing: observed_bool(
            execution,
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.passing,
            facts,
        )?,
        coverage_identity: observed_text(
            execution,
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.coverage_identity,
            facts,
        )?,
        proposal_identity,
        subject: EntityId::new(PartitionId::new(partition), slot, generation),
        source_identity,
        publication_identity,
        output_content_identity,
        program_revision,
    }))
}
