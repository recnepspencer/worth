//! Commit-boundary custody of published workflow definition members.

use std::num::NonZeroU64;
use std::sync::Arc;

use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
    CustomInvariantExecutionError, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantRuleId, CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
    StructuralReadError,
};
use worth_relational::facade::transactions::EntityReference;

use super::WorthQueryWorkflowLayout;

#[derive(Clone, Copy)]
struct WorkflowPublicationImmutability {
    entity_kinds: [KindId; 2],
    relation_kinds: [KindId; 5],
}

const RULE_ID: &str = "worth-query.workflow.publication-immutability";
const RULE_VERSION: CustomInvariantSemanticVersion = CustomInvariantSemanticVersion::new(1, 0);
const MAXIMUM_WORK_UNITS: u64 = 2_000_000;

pub(in crate::domain_computation::primary_graph) const fn publication_immutability_receipt_contract(
) -> (&'static str, u16, u16, u64) {
    (
        RULE_ID,
        RULE_VERSION.major,
        RULE_VERSION.minor,
        MAXIMUM_WORK_UNITS,
    )
}

pub(in crate::domain_computation::primary_graph) fn publication_immutability_registration(
    layout: &WorthQueryWorkflowLayout,
) -> Result<CustomInvariantRegistration, String> {
    CustomInvariantRegistration::new(WorkflowPublicationImmutability {
        entity_kinds: [layout.node.entity_kind, layout.connection.entity_kind],
        relation_kinds: [
            layout.definition_node_relation,
            layout.definition_connection_relation,
            layout.definition_start_relation,
            layout.connection_source_relation,
            layout.connection_target_relation,
        ],
    })
    .map_err(|error| format!("workflow publication immutability registration: {error:?}"))
}

impl CustomInvariantRule for WorkflowPublicationImmutability {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(RULE_ID),
                semantic_version: RULE_VERSION,
            },
            display_name: Arc::from("Workflow publication member immutability"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: NonZeroU64::new(MAXIMUM_WORK_UNITS)
                    .expect("positive finite work"),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: self.entity_kinds.to_vec(),
                    read_relation_kinds: self.relation_kinds.to_vec(),
                    affected_entity_kinds: self.entity_kinds.to_vec(),
                    affected_relation_kinds: self.relation_kinds.to_vec(),
                },
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if !context.touched().planned_entity_deletes().is_empty()
            || !context.touched().planned_relation_deletes().is_empty()
            || !context
                .touched()
                .planned_relation_endpoint_updates()
                .is_empty()
            || context
                .touched()
                .planned_relation_creates()
                .iter()
                .any(|create| {
                    matches!(create.source(), EntityReference::Existing(_))
                        || matches!(create.target(), EntityReference::Existing(_))
                })
        {
            return Ok(CustomInvariantVerdict::Violation);
        }
        for entity in context.touched().direct_visible_entity_ids() {
            let before = match context
                .committed_aspect_states()
                .entity_aspect_state(*entity)
            {
                Ok(before) => before,
                Err(StructuralReadError::RecordUnavailable) => continue,
                Err(error) => return Err(read_failure(error)),
            };
            let after = context
                .aspect_states()
                .entity_aspect_state(*entity)
                .map_err(read_failure)?;
            if before != after {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        for relation in context.touched().visible_relation_ids() {
            let before = match context.committed_relations().relation(*relation) {
                Ok(before) => before,
                // Adjacency expansion includes other relation kinds touching a
                // workflow member; those are not part of this rule's contract.
                Err(
                    StructuralReadError::RecordUnavailable
                    | StructuralReadError::OutsideDeclaredAccess,
                ) => continue,
                Err(error) => return Err(read_failure(error)),
            };
            let after = context
                .relations()
                .relation(*relation)
                .map_err(read_failure)?;
            if before != after {
                return Ok(CustomInvariantVerdict::Violation);
            }
            let before_fields = context
                .committed_aspect_states()
                .relation_aspect_state(*relation);
            let after_fields = context.aspect_states().relation_aspect_state(*relation);
            let unchanged_fields = match (before_fields, after_fields) {
                (Ok(before), Ok(after)) => before == after,
                (
                    Err(StructuralReadError::RecordUnavailable),
                    Err(StructuralReadError::RecordUnavailable),
                ) => true,
                (Err(StructuralReadError::RecordUnavailable), Ok(_))
                | (Ok(_), Err(StructuralReadError::RecordUnavailable)) => false,
                (Err(error), _) | (_, Err(error)) => return Err(read_failure(error)),
            };
            if !unchanged_fields {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}

fn read_failure(error: StructuralReadError) -> CustomInvariantExecutionError {
    CustomInvariantExecutionError::new(format!(
        "workflow publication immutability read failed: {error:?}"
    ))
}
