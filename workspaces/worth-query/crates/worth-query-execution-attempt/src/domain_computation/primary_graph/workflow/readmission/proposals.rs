//! Complete retained proposal coverage and captured request DATA, with no live scope.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::settlement::dependency,
    workflow::{
        captured_input::{inventory_identity, WorkflowCapturedTarget},
        proposal,
    },
};

pub(super) fn readmit(source: &WorkflowSourceRecords<'_, '_, '_>) -> Result<(), Denial> {
    let fields = &source.fields;
    let layout = source.layout;
    let population = &source.population;
    for proposal in population.entities(layout.proposal.entity_kind) {
        let proposal = *proposal;
        let transition = population.parent(layout.transition_proposal_relation, proposal)?;
        let instance = population.parent(layout.instance_transition_relation, transition)?;
        let definition = population
            .one(layout.instance_definition_relation, instance)?
            .ok_or_else(|| source.corrupt(instance, layout.instance.entity_kind))?;
        let data = source
            .definitions
            .get(&definition)
            .ok_or_else(|| source.corrupt(instance, layout.instance.entity_kind))?;
        let node = population
            .one(layout.transition_node_relation, transition)?
            .ok_or_else(|| source.corrupt(transition, layout.transition.entity_kind))?;
        let mut facts = Vec::new();
        proposal::observe_recorded_coverages(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            proposal,
            &mut facts,
        )
        .map_err(record_denial)?;
        let profile = proposal::read_recorded_captured_profile(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            proposal,
            &mut facts,
        )
        .map_err(record_denial)?;
        let mut expected = data.captured_targets(node).collect::<Vec<_>>();
        expected.sort_by(|left, right| left.path().cmp(right.path()));
        let members = population.outgoing(layout.captured_input.membership, proposal);
        let Some((digest, count)) = profile else {
            if !members.is_empty() || !expected.is_empty() {
                return Err(source.corrupt(proposal, layout.proposal.entity_kind));
            }
            continue;
        };
        if usize::try_from(count).ok() != Some(members.len()) || members.len() != expected.len() {
            return Err(source.corrupt(proposal, layout.proposal.entity_kind));
        }
        let identity = fields
            .text(
                proposal,
                layout.proposal.entity_kind,
                &layout.proposal.identity,
            )
            .map_err(Denial::Record)?;
        let identity = std::str::from_utf8(identity.bytes())
            .map_err(|_| source.corrupt(proposal, layout.proposal.entity_kind))?;
        let mut by_path = BTreeMap::new();
        for member in members {
            let path = fields
                .text(
                    *member,
                    layout.captured_input.entity_kind,
                    &layout.captured_input.node,
                )
                .map_err(Denial::Record)?;
            let path = std::str::from_utf8(path.bytes())
                .map_err(|_| source.corrupt(*member, layout.captured_input.entity_kind))?
                .to_owned();
            if by_path.insert(path, *member).is_some() {
                return Err(source.corrupt(*member, layout.captured_input.entity_kind));
            }
        }
        let mut targets = Vec::new();
        let mut resource = None;
        for node in expected {
            let member = *by_path
                .get(node.path())
                .ok_or_else(|| source.corrupt(proposal, layout.proposal.entity_kind))?;
            let contract = proposal::read_recorded_target(
                fields.execution,
                fields.runtime,
                fields.snapshot(),
                layout,
                proposal,
                identity,
                node.path(),
                member,
                &mut facts,
            )
            .map_err(record_denial)?;
            if !contract.matches_recorded_node(node)
                || resource.is_some_and(|previous| previous != contract.resource)
            {
                return Err(source.corrupt(member, layout.captured_input.entity_kind));
            }
            resource = Some(contract.resource);
            targets.push(WorkflowCapturedTarget {
                node: node.path().to_owned(),
                contract,
            });
        }
        if inventory_identity(&targets) != Some(digest) {
            return Err(source.corrupt(proposal, layout.proposal.entity_kind));
        }
    }
    for (kind, membership) in [
        (
            layout.captured_input.entity_kind,
            layout.captured_input.membership,
        ),
        (
            layout.proposal_coverage.entity_kind,
            layout.proposal_coverage_relation,
        ),
    ] {
        for child in population.entities(kind) {
            let proposal = population.parent(membership, *child)?;
            if !population.contains(layout.proposal.entity_kind, proposal) {
                return Err(source.corrupt(*child, kind));
            }
        }
    }
    for approval in population.entities(layout.approval.entity_kind) {
        source.attribute(
            *approval,
            source.revision(
                *approval,
                layout.approval.entity_kind,
                &layout.approval.program_revision,
            )?,
        )?;
    }
    for record in population.entities(layout.evidence_dependency.entity_kind) {
        let evidence = population.parent(layout.evidence_dependency_relation, *record)?;
        if !population.contains(layout.assessment_evidence.entity_kind, evidence) {
            return Err(source.corrupt(*record, layout.evidence_dependency.entity_kind));
        }
        dependency::observe_dependency(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            *record,
            &mut Vec::new(),
        )
        .map_err(record_denial)?;
    }
    Ok(())
}
fn record_denial(
    cause: crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
) -> Denial {
    Denial::Record(WorthQueryWorkflowReadDenial::Execution(cause))
}
