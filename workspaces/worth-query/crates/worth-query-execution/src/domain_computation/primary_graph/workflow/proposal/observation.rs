use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, observe_field_value, WorthQueryApplicationAdjacencyDirection,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

mod captured;
mod captured_history;
mod captured_inventory;
pub(in crate::domain_computation::primary_graph) use captured_history::observe as observe_captured_workflow_history;
mod coverage;
pub(in crate::domain_computation::primary_graph) use captured::target as observe_captured_workflow_input;
use coverage::observe_coverages;
pub(in crate::domain_computation::primary_graph) use coverage::observe_workflow_proposal_coverages;

pub(in crate::domain_computation::primary_graph) struct ObservedWorkflowOperationInput {
    pub(in crate::domain_computation::primary_graph) captured_identity: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) captured_count: Option<u64>,
    pub(in crate::domain_computation::primary_graph) input_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph) proposal: EntityId,
    pub(in crate::domain_computation::primary_graph) proposal_identity: String,
    pub(in crate::domain_computation::primary_graph) facts: Vec<WorthQueryApplicationObservedFact>,
}

pub(in crate::domain_computation::primary_graph) fn observe_workflow_operation_input(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    transition_identity: &str,
    expected_operation: &str,
    expected_input_type: &str,
    expected_node_path: &str,
    observation_request: &WorthQueryRequestScope,
) -> Result<ObservedWorkflowOperationInput, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let relations = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.transition_proposal_relation,
        transition,
        direction,
        2,
    )?
    .ok_or_else(|| denial("workflow operation input proposal is unavailable"))?;
    let [relation] = relations.as_slice() else {
        return Err(denial(
            "workflow operation input transition does not own exactly one proposal",
        ));
    };
    let proposal = relation.to;
    let kind = layout.proposal.entity_kind;
    let mut facts = vec![
        WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.transition_proposal_relation,
            anchor: transition,
            direction,
            maximum_work_units: 2,
            relations,
        },
        WorthQueryApplicationObservedFact::Entity {
            entity_id: proposal,
            kind,
        },
    ];
    let identity = required_text(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.identity,
        &mut facts,
    )?;
    let captured_profile =
        captured::profile(execution, runtime, snapshot, layout, proposal, &mut facts)?;
    let captured_identity = captured_profile.map(|profile| profile.0);
    let operation = required_text(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.operation,
        &mut facts,
    )?;
    let input_type = required_text(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.input_type,
        &mut facts,
    )?;
    let input_identity_text = required_text(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.input_identity,
        &mut facts,
    )?;
    let node_path = required_text(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.node_path,
        &mut facts,
    )?;
    let source_identity = optional_identity(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.source_identity,
        &mut facts,
    )?;
    let input_identity = decode_identity(&input_identity_text)
        .ok_or_else(|| denial("workflow operation input identity is malformed"))?;
    let coverage_count = required_u64(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.coverage_count,
        &mut facts,
    )?;
    let coverages = observe_coverages(
        execution,
        runtime,
        snapshot,
        layout,
        proposal,
        usize::try_from(coverage_count)
            .map_err(|_| denial("workflow proposal coverage count exceeds this host"))?,
        super::identity::WorkflowProposalCoverageScope::from_identities(
            &operation,
            &input_type,
            input_identity,
            source_identity,
        )
        .with_captured(captured_identity),
        &mut facts,
        observation_request,
    )?;
    let proposer = layout
        .proposal
        .proposer
        .observe(runtime, execution, snapshot, proposal, kind, &mut facts)?
        .ok_or_else(|| denial("current proposal lacks proposer"))?;
    let expected = super::derive_captured_workflow_proposal(
        transition_identity,
        &operation,
        &input_type,
        input_identity,
        source_identity,
        &node_path,
        coverages,
        proposer,
        captured_identity,
    );
    if identity != expected.identity()
        || operation != expected_operation
        || input_type != expected_input_type
        || node_path != expected_node_path
    {
        return Err(denial("workflow operation input proposal changed"));
    }
    check_request_live(observation_request, "workflow operation input")?;
    Ok(ObservedWorkflowOperationInput {
        captured_count: captured_profile.map(|profile| profile.1),
        captured_identity,
        input_identity,
        proposal,
        proposal_identity: identity,
        facts,
    })
}

pub(in crate::domain_computation::primary_graph) fn observe_workflow_proposal(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    expected: &super::WorkflowProposalMeaning,
    observation_request: &WorthQueryRequestScope,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let relations = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.transition_proposal_relation,
        transition,
        direction,
        2,
    )?
    .ok_or_else(|| denial("workflow proposal relation is unavailable"))?;
    let [relation] = relations.as_slice() else {
        return Err(denial(
            "workflow transition does not own exactly one proposal",
        ));
    };
    let proposal = relation.to;
    let kind = layout.proposal.entity_kind;
    let expected_identity =
        AspectValue::String(InternedString::Raw(expected.identity().to_owned()));
    let value = observe_field_value(
        runtime,
        execution,
        snapshot,
        proposal,
        kind,
        &layout.proposal.identity,
    )?
    .ok_or_else(|| denial("workflow proposal identity is unavailable"))?;
    if value != expected_identity {
        return Err(denial("workflow proposal identity changed"));
    }
    let mut facts = vec![
        WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: layout.transition_proposal_relation,
            anchor: transition,
            direction,
            maximum_work_units: 2,
            relations,
        },
        WorthQueryApplicationObservedFact::Entity {
            entity_id: proposal,
            kind,
        },
        WorthQueryApplicationObservedFact::Field {
            entity_id: proposal,
            kind,
            locator: layout.proposal.identity.clone(),
            value,
        },
    ];
    if captured::profile(execution, runtime, snapshot, layout, proposal, &mut facts)?
        .map(|profile| profile.0)
        != expected.captured_identity
    {
        return Err(denial("workflow proposal captured inventory changed"));
    }
    if layout
        .proposal
        .proposer
        .observe(runtime, execution, snapshot, proposal, kind, &mut facts)?
        != Some(expected.proposer)
    {
        return Err(denial(
            "current proposal proposer differs from original intent",
        ));
    }
    let coverage_count = required_u64(
        execution,
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.coverage_count,
        &mut facts,
    )?;
    if usize::try_from(coverage_count).ok() != Some(expected.coverages.len()) {
        return Err(denial("workflow proposal coverage count changed"));
    }
    let coverages = observe_coverages(
        execution,
        runtime,
        snapshot,
        layout,
        proposal,
        expected.coverages.len(),
        super::identity::WorkflowProposalCoverageScope::from_encoded(
            &expected.operation,
            &expected.input_type,
            &expected.input_identity,
            expected.source_identity.as_deref(),
        )
        .with_captured(expected.captured_identity),
        &mut facts,
        observation_request,
    )?;
    if coverages.as_slice() != expected.coverages.as_ref() {
        return Err(denial("workflow proposal coverage changed"));
    }
    check_request_live(observation_request, "workflow proposal")?;
    Ok(facts)
}

fn denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}

mod fields;
pub(in crate::domain_computation::primary_graph) use fields::decode_identity;
use fields::{optional_identity, required_text, required_u64};
