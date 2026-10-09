use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::{decode_identity, denial, optional_identity, required_text, required_u64};
use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    proposal::{identity::WorkflowProposalCoverageScope, WorkflowProposalCoverageMeaning},
    schema::WorthQueryWorkflowLayout,
};

pub(in crate::domain_computation::primary_graph) fn observe_workflow_proposal_coverages(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    observation_request: &WorthQueryRequestScope,
) -> Result<
    (
        Vec<WorkflowProposalCoverageMeaning>,
        Vec<WorthQueryApplicationObservedFact>,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    check_request_live(observation_request, "workflow coverage")?;
    let mut facts = Vec::new();
    let scope = observe_proposal_scope(runtime, snapshot, layout, proposal, &mut facts)?;
    let kind = layout.proposal.entity_kind;
    let count = required_u64(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.coverage_count,
        &mut facts,
    )?;
    let count = usize::try_from(count)
        .map_err(|_| denial("workflow proposal coverage count exceeds this host"))?;
    check_request_live(observation_request, "workflow coverage")?;
    let coverages = observe_coverages(
        runtime,
        snapshot,
        layout,
        proposal,
        count,
        scope,
        &mut facts,
        observation_request,
    )?;
    check_request_live(observation_request, "workflow coverage")?;
    Ok((coverages, facts))
}

fn observe_proposal_scope(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowProposalCoverageScope, WorthQueryApplicationAttemptDenial> {
    let kind = layout.proposal.entity_kind;
    let operation = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.operation,
        facts,
    )?;
    let input_type = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.input_type,
        facts,
    )?;
    let input_identity_text = required_text(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.input_identity,
        facts,
    )?;
    let input_identity = decode_identity(&input_identity_text)
        .ok_or_else(|| denial("workflow proposal input identity is malformed"))?;
    let source_identity = optional_identity(
        runtime,
        snapshot,
        proposal,
        kind,
        &layout.proposal.source_identity,
        facts,
    )?;
    Ok(WorkflowProposalCoverageScope::from_identities(
        &operation,
        &input_type,
        input_identity,
        source_identity,
    ))
}

pub(super) fn observe_coverages(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    count: usize,
    scope: WorkflowProposalCoverageScope,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
    observation_request: &WorthQueryRequestScope,
) -> Result<Vec<WorkflowProposalCoverageMeaning>, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    check_request_live(observation_request, "workflow proposal coverage")?;
    let maximum_work_units = count.checked_add(1).and_then(|count| count.checked_mul(2))
        .ok_or_else(|| crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Representability.into_attempt_denial("workflow proposal coverage"))?;
    let relations = observe_adjacency(
        runtime,
        snapshot,
        layout.proposal_coverage_relation,
        proposal,
        direction,
        maximum_work_units,
    )
    .ok_or_else(|| denial("workflow proposal coverage relation is unavailable"))?;
    if relations.len() != count {
        return Err(denial("workflow proposal coverage inventory changed"));
    }
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.proposal_coverage_relation,
        anchor: proposal,
        direction,
        maximum_work_units,
        relations: relations.clone(),
    });
    let mut coverages = Vec::with_capacity(count);
    for relation in relations {
        check_request_live(observation_request, "workflow proposal coverage")?;
        let entity = relation.to;
        let kind = layout.proposal_coverage.entity_kind;
        facts.push(WorthQueryApplicationObservedFact::Entity {
            entity_id: entity,
            kind,
        });
        let identity = required_text(
            runtime,
            snapshot,
            entity,
            kind,
            &layout.proposal_coverage.identity,
            facts,
        )?;
        let selector_text = required_text(
            runtime,
            snapshot,
            entity,
            kind,
            &layout.proposal_coverage.selector,
            facts,
        )?;
        let selector =
            ApplicationWorkflowSubjectSelector::from_persistence_identity(&selector_text)
                .ok_or_else(|| denial("workflow proposal coverage selector is malformed"))?;
        let partition = required_u64(
            runtime,
            snapshot,
            entity,
            kind,
            &layout.proposal_coverage.subject_partition,
            facts,
        )?;
        let slot = required_u64(
            runtime,
            snapshot,
            entity,
            kind,
            &layout.proposal_coverage.subject_slot,
            facts,
        )?;
        let generation = required_u64(
            runtime,
            snapshot,
            entity,
            kind,
            &layout.proposal_coverage.subject_generation,
            facts,
        )?;
        let generation = u32::try_from(generation)
            .map_err(|_| denial("workflow proposal coverage generation is malformed"))?;
        let partition = u32::try_from(partition)
            .map_err(|_| denial("workflow proposal coverage partition is malformed"))?;
        let mut coverage = WorkflowProposalCoverageMeaning::new(
            selector,
            EntityId::new(PartitionId::new(partition), slot, generation),
        );
        coverage.identity = scope.coverage_identity(coverage.local_identity());
        if coverage.identity != identity {
            return Err(denial("workflow proposal coverage identity changed"));
        }
        coverages.push(coverage);
    }
    coverages.sort_by(|left, right| left.selector.cmp(&right.selector));
    if coverages
        .windows(2)
        .any(|pair| pair[0].selector == pair[1].selector)
    {
        return Err(denial("workflow proposal coverage selector is duplicated"));
    }
    Ok(coverages)
}
