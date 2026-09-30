use super::*;
use crate::facade::runtime::{
    CustomInvariantExecutionContext, CustomInvariantRule, CustomInvariantScopePlanner,
};
use crate::identity::data::EntityId;
use crate::validation::data::*;
use std::sync::Arc;
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};

pub(super) struct RequiredChild {
    pub(super) execution_point: InvariantExecutionPoint,
    pub(super) identity: &'static str,
    pub(super) evaluations: Arc<AtomicUsize>,
}

impl CustomInvariantRule for RequiredChild {
    type Scope = Vec<EntityId>;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(self.identity),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Required child completeness"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4096).unwrap(),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1), KindId(3)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: vec![KindId(1), KindId(3)],
                    affected_relation_kinds: vec![KindId(2)],
                    include_relation_endpoint_entity_touches: true,
                },
                execution_point: self.execution_point,
                groups: InvariantGroupSet::of(
                    if self.execution_point == InvariantExecutionPoint::SnapshotPublication {
                        InvariantGroup::PublicationCoherence
                    } else {
                        InvariantGroup::SchemaCompliance
                    },
                ),
                cost_class: InvariantCostClass::Touched,
                failure_effect: if self.execution_point
                    == InvariantExecutionPoint::SnapshotPublication
                {
                    InvariantFailureEffect::BlockPublication
                } else {
                    InvariantFailureEffect::BlockCommit
                },
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let relations = planner.relations();
        let mut children = Vec::new();
        for root in planner.touched().visible_entity_ids() {
            if relations.readable_entity_kind(*root).ok().flatten() != Some(KindId(1)) {
                continue;
            }
            let edges = relations.outgoing_relations_for_entity(*root)?;
            if edges.len() != 1 {
                return Err(CustomInvariantPreparationError::new(
                    "retained root lacks its required child",
                ));
            }
            let edge = relations.relation(edges[0]).map_err(|error| {
                CustomInvariantPreparationError::new(format!(
                    "required edge unavailable: {error:?}"
                ))
            })?;
            if relations.entity_kind(edge.target).map_err(|error| {
                CustomInvariantPreparationError::new(format!(
                    "required child unavailable: {error:?}"
                ))
            })? != KindId(3)
            {
                return Err(CustomInvariantPreparationError::new(
                    "required child has foreign kind",
                ));
            }
            planner
                .aspect_states()
                .entity_aspect_state(edge.target)
                .map_err(|error| {
                    CustomInvariantPreparationError::new(format!(
                        "required child value unavailable: {error:?}"
                    ))
                })?;
            children.push(edge.target);
        }
        Ok(children)
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        children: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        for child in children {
            let state = context
                .aspect_states()
                .entity_aspect_state(*child)
                .map_err(|error| {
                    CustomInvariantExecutionError::new(format!(
                        "required child value unavailable: {error:?}"
                    ))
                })?;
            let valid = state.get(&crate::tests::support::aspect_key("valid"));
            if !valid.is_some_and(|value| {
                matches!(
                    value.view(),
                    ContractValidatedAspectValueView::Scalar(AspectValue::Bool(true))
                )
            }) {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        if !children.is_empty() {
            self.evaluations.fetch_add(1, Ordering::SeqCst);
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}

pub(super) fn deny_invalid_child_value(
    runtime: &crate::runtime::RelationalRuntime,
    child: EntityId,
) {
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(WorkerIntentBatch::new("invalidate-required-child").push(
            MutationIntent::Entity(
                crate::transactions::data::EntityMutationIntent::UpdateFields(
                    crate::transactions::data::UpdateEntityFieldsIntent {
                        entity_id: child,
                        fields: valid_field_patch(false),
                    },
                ),
            ),
        ))
        .unwrap();
    let denial = transaction
        .commit(runtime)
        .expect_err("ordinary invalid child value is rejected");
    assert_custom_violation(&denial);
}

pub(super) fn assert_custom_violation(error: &crate::transactions::data::TransactionCommitError) {
    use crate::transactions::data::{ConflictClass, TransactionCommitError};
    assert!(
        matches!(error,
            TransactionCommitError::Conflict { error, .. }
            if matches!(&error.class, ConflictClass::InvariantViolation {
                fields: InvariantViolationFields::CustomInvariantViolation { identity }, ..
            } if ["test.retained-root.child-completeness.commit",
                "test.retained-root.child-completeness.mutation",
                "test.retained-root.child-completeness.publication"].contains(&identity.rule_id.as_str()))
        ),
        "expected typed required-child invariant violation, got {error:?}"
    );
}
