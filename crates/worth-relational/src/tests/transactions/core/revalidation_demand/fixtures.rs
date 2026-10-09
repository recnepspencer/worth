//! The strictness rule these courts judge records with.
//!
//! The rule stands in for the Query program whose adoption motivates the
//! demand: it reads the *proposed* state of the records a candidate touches,
//! and whether a record passes depends on a mode carried by a different record
//! in the same candidate. That is the whole shape of "existing state must be
//! judged under the meaning this candidate proposes".

use std::sync::Arc;

use crate::tests::support::*;
use crate::validation::data::{
    CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
    CustomInvariantExecutionError, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantRuleId, CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
};
use worth_foundational::facade::{AspectKey, AspectValue, ContractValidatedAspectValueView};

/// The rule's own identifier, so a court can name the rule that rejected.
pub(super) const STRICTNESS_RULE_ID: &str = "test.revalidation.strictness";

/// A record carrying this name puts every other touched record under the strict
/// reading. It is the analogue of the branch's active program.
pub(super) const STRICT_MODE_NAME: &str = "strict";

/// The only name the strict reading accepts.
pub(super) const COMPLIANT_NAME: &str = "ok";

pub(super) struct StrictnessRule;

impl CustomInvariantRule for StrictnessRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(STRICTNESS_RULE_ID),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Test Revalidation Strictness"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(4_096).unwrap(),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: Vec::new(),
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: Vec::new(),
                    include_relation_endpoint_entity_touches: true,
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
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let touched = context.touched().visible_entity_ids().to_vec();
        let states = context.aspect_states();
        let mut names = Vec::with_capacity(touched.len());
        for entity_id in touched {
            names.push(proposed_name(&states, entity_id)?);
        }
        if !names
            .iter()
            .any(|name| name == &name_value(STRICT_MODE_NAME))
        {
            return Ok(CustomInvariantVerdict::Pass);
        }
        if names.iter().all(|name| {
            name == &name_value(STRICT_MODE_NAME) || name == &name_value(COMPLIANT_NAME)
        }) {
            Ok(CustomInvariantVerdict::Pass)
        } else {
            Ok(CustomInvariantVerdict::Violation)
        }
    }
}

/// The stored shape of one `name` field value, so a court and the rule agree on
/// what they are comparing without either reaching into string interning.
pub(super) fn name_value(name: &str) -> AspectValue {
    AspectValue::String(name.into())
}

fn proposed_name(
    states: &crate::validation::data::StructuralAspectStateView<'_>,
    entity_id: crate::facade::identity::EntityId,
) -> Result<AspectValue, CustomInvariantExecutionError> {
    let state = states.entity_aspect_state(entity_id).map_err(|error| {
        CustomInvariantExecutionError::new(format!("structural read refused: {error:?}"))
    })?;
    let Some(value) = state.get(&AspectKey::new("name").expect("valid test aspect key")) else {
        return Ok(AspectValue::Null);
    };
    match value.view() {
        ContractValidatedAspectValueView::Scalar(scalar) => Ok(scalar.clone()),
        _ => Ok(AspectValue::Null),
    }
}

/// A runtime whose only custom rule is the strictness rule.
pub(super) fn strictness_runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .custom_invariant(CustomInvariantRegistration::new(StrictnessRule).unwrap())
        .build()
}

/// One batch carrying a revalidation demand per record, in the order given.
pub(super) fn revalidation_batch(
    label: &str,
    entity_ids: impl IntoIterator<Item = crate::facade::identity::EntityId>,
) -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new(label);
    for entity_id in entity_ids {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
            crate::transactions::data::RevalidateEntityIntent { entity_id },
        )));
    }
    batch
}
