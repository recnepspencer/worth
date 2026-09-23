use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;

use super::super::fact::{observe_adjacency, observe_indexed_entity_selection};
use super::super::{
    WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationObservedFact,
};
use super::WorkflowDefinitionExpectedPredecessor;
use crate::domain_computation::primary_graph::workflow::definition::WorkflowLineagePublicationTarget;
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

const LINEAGE_LOOKUP_LIMIT: usize = 2;
const CURRENT_DEFINITION_WORK_LIMIT: usize = 2;

pub(super) struct SelectedLineage {
    pub(super) target: WorkflowLineagePublicationTarget,
    pub(super) facts: Vec<WorthQueryApplicationObservedFact>,
}

pub(super) fn select_lineage<Spec: ApplicationWorkflowSpec>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: &worth_query_declaration::facade::application_program::ValidatedWorkflowDefinition<
        Spec,
    >,
    expected_predecessor: &WorkflowDefinitionExpectedPredecessor,
) -> Result<SelectedLineage, WorthQueryApplicationAttemptDenial> {
    let identity = text(definition.identity().as_str());
    let selection = observe_indexed_entity_selection(
        runtime,
        snapshot,
        layout.lineage.identity_index_id,
        layout.lineage.entity_kind,
        layout.lineage.identity.clone(),
        identity,
        LINEAGE_LOOKUP_LIMIT,
    )
    .ok_or_else(|| lineage_denial(definition.identity().as_str()))?;
    let candidates = match &selection {
        WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } => {
            candidates.clone()
        }
        _ => unreachable!("lineage selection owner returns its exact fact kind"),
    };
    if candidates.len() > 1 {
        return Err(lineage_denial(definition.identity().as_str()));
    }
    let mut facts = vec![selection];
    let Some(lineage) = candidates.first().copied() else {
        if expected_predecessor.definition_entity().is_some() {
            facts.push(
                WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor {
                    relation_kind: layout.current_definition_relation,
                    lineage: None,
                    expected_definition: expected_predecessor.definition_entity(),
                    maximum_work_units: CURRENT_DEFINITION_WORK_LIMIT,
                },
            );
        }
        return Ok(SelectedLineage {
            target: WorkflowLineagePublicationTarget::New,
            facts,
        });
    };

    facts.push(exact_field_fact(
        runtime,
        snapshot,
        lineage,
        layout.lineage.entity_kind,
        layout.lineage.spec.clone(),
        text(Spec::IDENTITY.as_str()),
        definition.identity().as_str(),
    )?);
    facts.push(exact_field_fact(
        runtime,
        snapshot,
        lineage,
        layout.lineage.entity_kind,
        layout.lineage.protocol_version.clone(),
        AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        definition.identity().as_str(),
    )?);
    let relations = observe_adjacency(
        runtime,
        snapshot,
        layout.current_definition_relation,
        lineage,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        CURRENT_DEFINITION_WORK_LIMIT,
    )
    .ok_or_else(|| lineage_denial(definition.identity().as_str()))?;
    let [current] = relations.as_slice() else {
        return Err(lineage_denial(definition.identity().as_str()));
    };
    let current = *current;
    facts.push(
        WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor {
            relation_kind: layout.current_definition_relation,
            lineage: Some(lineage),
            expected_definition: expected_predecessor.definition_entity(),
            maximum_work_units: CURRENT_DEFINITION_WORK_LIMIT,
        },
    );
    let current_definition = WorthQueryApplicationObservedFact::Entity {
        entity_id: current.to,
        kind: layout.definition.entity_kind,
    };
    if !current_definition.remains_equal_in(runtime, snapshot) {
        return Err(lineage_denial(definition.identity().as_str()));
    }
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.current_definition_relation,
        anchor: lineage,
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: CURRENT_DEFINITION_WORK_LIMIT,
        relations,
    });
    facts.push(current_definition);
    Ok(SelectedLineage {
        target: WorkflowLineagePublicationTarget::Existing {
            lineage,
            current_relation: current.relation_id,
        },
        facts,
    })
}

fn exact_field_fact(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity_id: worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: worth_foundational::facade::AspectFieldLocator,
    expected: AspectValue,
    subject: &str,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    if super::super::observation::observe_field_value(runtime, snapshot, entity_id, kind, &locator)
        .as_ref()
        != Some(&expected)
    {
        return Err(lineage_denial(subject));
    }
    Ok(WorthQueryApplicationObservedFact::Field {
        entity_id,
        kind,
        locator,
        value: expected,
    })
}

fn lineage_denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowLineageUnavailable,
        subject,
    )
}

fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
