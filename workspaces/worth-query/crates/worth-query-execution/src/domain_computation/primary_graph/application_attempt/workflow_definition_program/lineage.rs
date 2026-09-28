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
pub(super) const CURRENT_DEFINITION_WORK_LIMIT: usize = 2;

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

    let spec_identity = Spec::IDENTITY;
    facts.extend(lineage_identity_facts(
        runtime,
        snapshot,
        layout,
        lineage,
        spec_identity.as_str(),
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
    // A retired lineage has no current relation. The predecessor fact decides
    // whether that absence is the author's expectation or a stale revision.
    let current = match relations.as_slice() {
        [] => None,
        [current] => Some(*current),
        _ => return Err(lineage_denial(definition.identity().as_str())),
    };
    facts.push(
        WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor {
            relation_kind: layout.current_definition_relation,
            lineage: Some(lineage),
            expected_definition: expected_predecessor.definition_entity(),
            maximum_work_units: CURRENT_DEFINITION_WORK_LIMIT,
        },
    );
    let current_definition = current.map(|current| WorthQueryApplicationObservedFact::Entity {
        entity_id: current.to,
        kind: layout.definition.entity_kind,
    });
    if current_definition
        .as_ref()
        .is_some_and(|fact| !fact.remains_equal_in(runtime, snapshot))
    {
        return Err(lineage_denial(definition.identity().as_str()));
    }
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.current_definition_relation,
        anchor: lineage,
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: CURRENT_DEFINITION_WORK_LIMIT,
        relations,
    });
    facts.extend(current_definition);
    Ok(SelectedLineage {
        target: WorkflowLineagePublicationTarget::Existing {
            lineage,
            current_relation: current.map(|current| current.relation_id),
        },
        facts,
    })
}

/// The lineage's spec and fact-protocol fields, exactly as publication wrote
/// them. Every lineage reader pins both before trusting its relations.
pub(super) fn lineage_identity_facts(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    lineage: worth_relational::facade::identity::EntityId,
    spec_identity: &str,
    subject: &str,
) -> Result<[WorthQueryApplicationObservedFact; 2], WorthQueryApplicationAttemptDenial> {
    Ok([
        exact_field_fact(
            runtime,
            snapshot,
            lineage,
            layout.lineage.entity_kind,
            layout.lineage.spec.clone(),
            text(spec_identity),
            subject,
        )?,
        exact_field_fact(
            runtime,
            snapshot,
            lineage,
            layout.lineage.entity_kind,
            layout.lineage.protocol_version.clone(),
            AspectValue::UInt64(
                crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
            ),
            subject,
        )?,
    ])
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

pub(super) fn lineage_denial(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowLineageUnavailable,
        subject,
    )
}

pub(super) fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
