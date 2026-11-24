//! Schema and custom rule used to exercise native staged validation evidence.
use crate::identity::data::KindId;
use crate::schema::data::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use crate::validation::{
    custom_rule::{
        CustomInvariantExecutionContext, CustomInvariantLeaseBudget, CustomInvariantRule,
        CustomInvariantScopePlanner,
    },
    data::{
        custom_rule::errors::{
            CustomInvariantExecutionError, CustomInvariantPreparationError, CustomInvariantVerdict,
        },
        descriptor::{
            CustomInvariantAccessContract, CustomInvariantDescriptor,
            CustomInvariantOperationalMetadata,
        },
        execution::{InvariantExecutionPoint, InvariantFailureEffect},
        groups::{InvariantCostClass, InvariantGroupSet},
        rule_id::{
            CustomInvariantRuleId, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
        },
    },
};
use std::{num::NonZeroU64, sync::Arc};

pub(super) const UNRELATED: KindId = KindId(1);
const GOVERNED: KindId = KindId(2);

pub(super) struct ClosureRule;
impl CustomInvariantRule for ClosureRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("basis.closure"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Closure rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(NonZeroU64::new(1_000_000).unwrap()),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![GOVERNED],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![GOVERNED],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: false,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                // Every group, so plan-contract group selection never hides the
                // rule: these courts judge only the closure basis.
                groups: InvariantGroupSet::all(),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn prepare_scope_checked(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        Some(Ok(()))
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
    fn evaluate_checked(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(Ok(CustomInvariantVerdict::Pass))
    }
}

pub(super) fn schema() -> RelationalSchemaRegistry {
    let mut schema = RelationalSchemaRegistry::new();
    for (kind, name) in [(UNRELATED, "unrelated"), (GOVERNED, "governed")] {
        schema = schema
            .register_entity_kind(EntityKindRegistration {
                kind_id: kind,
                kind_name: name.into(),
                schema_id: SchemaId("basis".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .unwrap();
    }
    schema
}
