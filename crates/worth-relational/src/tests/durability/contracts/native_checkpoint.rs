use super::*;
use std::sync::Arc;
mod speculative_symbols;

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
        .native_checkpoint_with_policy(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let mut recovered = memory_runtime_with_builder_invariant();

    recovered
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &checkpoint,
            crate::tests::support::durability::captured_members_source(&checkpoint),
        )
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

/// Fixture adaptation (C2.3): the original ran on, and restored into, runtimes
/// bound to the retired in-crate test medium. The captures are Embedded and the
/// claim is the owner's internal checkpoint retention, so it has no
/// physical-medium content and runs on the Memory runtime.
#[test]
fn repeated_native_captures_do_not_retain_checkpoint_images() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "native-checkpoint-first");
    let retained_before = runtime.durability.checkpoints().len();

    for label in ["second", "third", "fourth"] {
        runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: None,
                },
            )
            .expect("each settled world encodes without retaining an internal checkpoint");
        create_entity_outcome(execution, &runtime, label);
    }
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .expect("the latest settled world encodes");

    assert_eq!(runtime.durability.checkpoints().len(), retained_before);
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &checkpoint,
            crate::tests::support::durability::captured_members_source(&checkpoint),
        )
        .expect("the final non-retained checkpoint restores");
    create_entity_outcome(execution, &recovered, "post-bounded-retention-edit");
}

fn memory_runtime_with_builder_invariant() -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .custom_invariant(CustomInvariantRegistration::new(NativeCheckpointRule).unwrap())
        .build()
        .expect("valid native runtime configuration")
}

#[test]
fn corrupt_native_checkpoint_is_denied_without_publishing_recovered_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut bytes = crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC.to_vec();
    bytes.extend([0xc1, 0x00, 0xff]);
    let corrupt = crate::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes(
        bytes.into_boxed_slice(),
    );
    let medium = OwnedTestDurableMedium::new();
    let mut recovered = durable_runtime_for_medium(&medium);
    let error = recovered
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &corrupt,
            crate::tests::support::durability::captured_members_source(&corrupt),
        )
        .expect_err("corrupt native bytes cannot become runtime authority");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}
