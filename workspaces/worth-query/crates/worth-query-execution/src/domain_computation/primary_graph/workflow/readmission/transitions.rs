//! Original transition/approval/proposal correspondence, never progression or replay retention.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::{history_projection,history::operation_outputs,settlement};

pub(super) fn readmit(source: &WorkflowSourceRecords<'_, '_, '_>) -> Result<(), Denial> {
    let fields = &source.fields;
    let layout = source.layout;
    let population = &source.population;
    let branch = crate::basis::WorthQueryProductBranch::from_occurrence(
        source.branch.lifecycle_incarnation(),
    );
    for transition in population.entities(layout.transition.entity_kind) {
        let transition = *transition;
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
        if data.node(node).is_none() {
            return Err(source.corrupt(transition, layout.transition.entity_kind));
        }
        source.revision(
            transition,
            layout.transition.entity_kind,
            &layout.transition.identity,
        )?;
        history_projection::observe(
            fields,
            layout,
            branch,
            instance,
            definition,
            transition,
            true,
            usize::MAX,
        )
        .map_err(Denial::Record)?;
        operation_outputs::read_recorded(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            transition,
        )
        .map_err(|cause| Denial::Record(WorthQueryWorkflowReadDenial::Execution(cause)))?;
        if let Some(evidence) = settlement::observe_assessment_evidence(
            fields.execution,
            fields.runtime,
            fields.snapshot(),
            layout,
            transition,
            &mut Vec::new(),
        )
        .map_err(|cause| Denial::Record(WorthQueryWorkflowReadDenial::Execution(cause)))?
        {
            let revision = CanonicalDigestId::parse_hex(&evidence.program_revision)
                .filter(|revision| revision.render_hex() == evidence.program_revision)
                .ok_or_else(|| source.corrupt(transition, layout.transition.entity_kind))?;
            source.attribute(transition, revision)?;
        }
    }
    // Every immutable child must belong to an actual transition on this source.
    for (kind, membership) in [
        (
            layout.operation_output.entity_kind,
            layout.operation_output.membership,
        ),
        (
            layout.assessment_evidence.entity_kind,
            layout.transition_assessment_evidence_relation,
        ),
        (
            layout.approval.entity_kind,
            layout.transition_approval_relation,
        ),
    ] {
        for child in population.entities(kind) {
            let parent = population.parent(membership, *child)?;
            if !population.contains(layout.transition.entity_kind, parent) {
                return Err(source.corrupt(*child, kind));
            }
        }
    }
    for instance in population.entities(layout.instance.entity_kind) {
        for transition in population.outgoing(layout.instance_prior_effect_relation, *instance) {
            let ancestor = population.parent(layout.instance_transition_relation, *transition)?;
            if !is_migration_ancestor(source, *instance, ancestor)? {
                return Err(source.corrupt(*instance, layout.instance.entity_kind));
            }
            let receipt = fields
                .optional_text(
                    *transition,
                    layout.transition.entity_kind,
                    &layout.transition.operation_receipt_identity,
                )
                .map_err(Denial::Record)?;
            if receipt.is_none() {
                return Err(source.corrupt(*transition, layout.transition.entity_kind));
            }
        }
    }
    Ok(())
}
fn is_migration_ancestor(
    source: &WorkflowSourceRecords<'_, '_, '_>,
    instance: EntityId,
    wanted: EntityId,
) -> Result<bool, Denial> {
    let mut cursor = instance;
    let mut visited = BTreeSet::new();
    while let Some(parent) = source
        .population
        .one(source.layout.instance_migrated_from_relation, cursor)?
    {
        if !visited.insert(parent) {
            return Err(source.corrupt(instance, source.layout.instance.entity_kind));
        }
        if parent == wanted {
            return Ok(true);
        }
        cursor = parent;
    }
    Ok(false)
}
