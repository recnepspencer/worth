use worth_query_declaration::facade::application_program::ApplicationWorkflowSubjectSelector;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::{denial, required_text, required_u64};
use crate::domain_computation::primary_graph::application_attempt::{
    observe_adjacency, WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    proposal::WorkflowProposalCoverageMeaning, schema::WorthQueryWorkflowLayout,
};

pub(in crate::domain_computation::primary_graph) fn observe_workflow_proposal_coverage(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    selector: &ApplicationWorkflowSubjectSelector,
) -> Result<
    (
        WorkflowProposalCoverageMeaning,
        Vec<WorthQueryApplicationObservedFact>,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let mut facts = Vec::new();
    let count = required_u64(
        runtime,
        snapshot,
        proposal,
        layout.proposal.entity_kind,
        &layout.proposal.coverage_count,
        &mut facts,
    )?;
    let coverages = observe_coverages(
        runtime,
        snapshot,
        layout,
        proposal,
        usize::try_from(count)
            .map_err(|_| denial("workflow proposal coverage count exceeds this host"))?,
        &mut facts,
    )?;
    let matching = coverages
        .into_iter()
        .filter(|coverage| &coverage.selector == selector)
        .collect::<Vec<_>>();
    match matching.as_slice() {
        [coverage] => Ok((coverage.clone(), facts)),
        _ => Err(denial(
            "workflow proposal does not contain exactly one required coverage",
        )),
    }
}

pub(super) fn observe_coverages(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    count: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<WorkflowProposalCoverageMeaning>, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Outgoing;
    let maximum_work_units = count.saturating_add(1).saturating_mul(2);
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
        let coverage = WorkflowProposalCoverageMeaning::new(
            selector,
            EntityId::new(PartitionId::new(partition), slot, generation),
        );
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
