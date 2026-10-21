//! A real relation-only rule, governed by the proposed source endpoint's name.
use super::fixtures::{COMPLIANT_NAME, STRICT_MODE_NAME};
use crate::tests::support::*;
use crate::validation::data::*;
use std::sync::Arc;
use worth_foundational::facade::{AspectKey, AspectValue, ContractValidatedAspectValueView};

pub(super) const RULE: &str = "test.revalidation.relation-label";
struct RelationLabelRule;
impl CustomInvariantRule for RelationLabelRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(RULE),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Relation label under proposed source mode"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(4096).unwrap()),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: vec![],
                    affected_relation_kinds: vec![KindId(2)],
                    include_relation_endpoint_entity_touches: false,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        for id in context.touched().visible_relation_ids() {
            let relation = context.relations().relation(*id).map_err(read_error)?;
            assert!(
                context
                    .touched()
                    .direct_visible_entity_ids()
                    .contains(&relation.source),
                "a relation-only rule retains its actual source endpoint"
            );
            assert!(
                context
                    .touched()
                    .direct_visible_entity_ids()
                    .contains(&relation.target),
                "a relation-only rule retains its actual target endpoint"
            );
            let states = context.aspect_states();
            let source = states
                .entity_aspect_state(relation.source)
                .map_err(read_error)?;
            let value = states.relation_aspect_state(*id).map_err(read_error)?;
            if scalar(source, "name") == Some(AspectValue::String(STRICT_MODE_NAME.into()))
                && scalar(value, "label") != Some(AspectValue::String(COMPLIANT_NAME.into()))
            {
                return Ok(CustomInvariantVerdict::Violation);
            }
        }
        Ok(CustomInvariantVerdict::Pass)
    }
}
fn read_error(error: StructuralReadError) -> CustomInvariantExecutionError {
    CustomInvariantExecutionError::new(format!("native structural read: {error:?}"))
}
fn scalar(
    value: &worth_foundational::facade::AuthoritativeRecordAspectState,
    key: &str,
) -> Option<AspectValue> {
    match value.get(&AspectKey::new(key).unwrap())?.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value.clone()),
        _ => None,
    }
}
pub(super) fn runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .custom_invariant(CustomInvariantRegistration::new(RelationLabelRule).unwrap())
        .build()
}
pub(super) fn batch(id: RelationId) -> WorkerIntentBatch {
    WorkerIntentBatch::new("judge-unchanged-relation").push(MutationIntent::Relation(
        RelationMutationIntent::Revalidate(crate::facade::transactions::RevalidateRelationIntent {
            relation_id: id,
        }),
    ))
}
pub(super) fn assert_rule(error: TransactionCommitError) {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("expected native rule conflict")
    };
    assert!(
        matches!(error.class, crate::transactions::data::ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::CustomInvariantViolation { ref identity }, ..
    } if identity.rule_id.as_str() == RULE),
        "actual refusal: {:?}",
        error.class
    );
}
