//! Owner-truth writes that realize admitted workflow dispositions inside the
//! program adoption transaction.
//!
//! Carry rebinds the exact instance to the target revision in place, so no
//! global instance revision moves and no approval, evidence, or receipt is
//! copied. A carried definition keeps the revision it was published under and
//! records the carriage beside it. A carried instance also carries the
//! definition it is pinned to, current or retired, because execution checks
//! the definition header against the same revision. Retire removes only the
//! current-definition relation. Cancel ends the instance and drops its live
//! membership.

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{
    AspectFieldPatch, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    RelationMutationIntent, UpdateEntityFieldsIntent,
};

use super::super::instance::WorkflowInstanceState;
use super::super::schema::WorthQueryWorkflowLayout;
use super::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDefinitionDisposition,
    WorthQueryWorkflowDispositions, WorthQueryWorkflowInstanceDisposition,
};
use crate::domain_computation::primary_graph::program_occurrence::program_revision_rendering;

/// Intents for already-admitted `choices`; see
/// [`WorthQueryWorkflowAdoptionInventory::admit`].
pub(in crate::domain_computation::primary_graph) fn stage_workflow_dispositions(
    layout: &WorthQueryWorkflowLayout,
    inventory: &WorthQueryWorkflowAdoptionInventory,
    choices: &WorthQueryWorkflowDispositions,
    target: &ApplicationProgramRevision,
) -> Vec<MutationIntent> {
    let revision = program_revision_rendering(target);
    let mut intents = Vec::new();
    let mut rebound_definitions = BTreeSet::<EntityId>::new();
    for occurrence in inventory.definitions() {
        match choices.definition_disposition(occurrence.entity_id()) {
            Some(WorthQueryWorkflowDefinitionDisposition::Carry) => {
                rebound_definitions.insert(occurrence.entity_id());
            }
            // The same single effect as a standalone retirement: only the
            // current-definition relation goes; history stays.
            Some(WorthQueryWorkflowDefinitionDisposition::Retire) => {
                intents.push(delete_relation(occurrence.current_relation));
            }
            None => {}
        }
    }
    for occurrence in inventory.instances() {
        match choices.instance_disposition(occurrence.entity_id()) {
            Some(WorthQueryWorkflowInstanceDisposition::Carry) => {
                rebound_definitions.insert(occurrence.definition());
                intents.push(update_field(
                    occurrence.entity_id(),
                    &layout.instance.program_revision,
                    revision.clone(),
                ));
            }
            Some(WorthQueryWorkflowInstanceDisposition::Cancel) => {
                intents.push(update_field(
                    occurrence.entity_id(),
                    &layout.instance.state,
                    AspectValue::UInt64(WorkflowInstanceState::Cancelled.persisted_tag()),
                ));
                intents.push(delete_relation(occurrence.live_membership));
            }
            None => {}
        }
    }
    for definition in rebound_definitions {
        intents.push(update_field(
            definition,
            &layout.definition.carried_revision,
            revision.clone(),
        ));
    }
    intents
}

fn update_field(
    entity_id: EntityId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    value: AspectValue,
) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id,
            fields: AspectFieldPatch::from(BTreeMap::from([(locator.clone(), value)])),
        },
    ))
}

fn delete_relation(relation_id: worth_relational::facade::identity::RelationId) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id,
    }))
}
