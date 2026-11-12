use super::*;
use std::sync::Arc;

use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
    CustomInvariantScopePlanner, CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
    CustomInvariantVerdict, InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    InvariantGroup, InvariantGroupSet,
};

struct NativeCheckpointRule;

impl CustomInvariantRule for NativeCheckpointRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("native.checkpoint.rule"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Native Checkpoint Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(u64::MAX).unwrap()),
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
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn native_checkpoint_preserves_unsealed_builder_invariant_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = memory_runtime_with_builder_invariant();
    create_entity_outcome(execution, &source, "native-checkpoint-builder-invariant");
    let checkpoint = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut recovered = memory_runtime_with_builder_invariant();

    recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .expect("native recovery preserves target runtime extensions");

    assert_eq!(
        recovered
            .schema_contract_runtime
            .custom_invariant_registries
            .iter()
            .next()
            .unwrap()
            .rule_id()
            .as_str(),
        "native.checkpoint.rule"
    );
    assert!(
        !recovered
            .schema_contract_runtime
            .initial_custom_invariants_sealed
    );
}

fn memory_runtime_with_builder_invariant() -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .custom_invariant(CustomInvariantRegistration::new(NativeCheckpointRule).unwrap())
        .build()
        .expect("valid native runtime configuration")
}
