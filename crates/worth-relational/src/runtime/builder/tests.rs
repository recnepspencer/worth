use std::sync::Arc;

use super::RelationalRuntimeBuilder;
use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

struct BuilderTestRule;

impl CustomInvariantRule for BuilderTestRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("builder.test.rule"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Builder Test Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1).unwrap()),
                access: crate::validation::data::CustomInvariantAccessContract::default(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
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
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn builder_attaches_custom_invariants_without_polluting_config() {
    let registration = CustomInvariantRegistration::new(BuilderTestRule).unwrap();
    let runtime = RelationalRuntimeBuilder::new()
        .custom_invariant(registration)
        .build()
        .expect("valid native runtime configuration");

    assert_eq!(
        runtime.config.schema.invariant_catalog.registrations.len(),
        2
    );
    assert_eq!(
        runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .len(),
        1
    );
    assert_eq!(
        runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .next()
            .unwrap()
            .rule_id()
            .as_str(),
        "builder.test.rule"
    );
}

mod construction_refusals;
