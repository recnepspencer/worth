use std::sync::Arc;

use super::{
    StrategyCommitArtifactBundle, StrategyIntentScopeDigest, StrategyMergeConflictClass,
    StrategyMergeDescriptor, StrategyMergeSemantics,
};
use crate::capabilities::{RuntimeConfigSource, SchemaSource};
use crate::commit_strategies::data::canonical_digest::native_entity_fields_scope_digest;
use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact, CanonicalStrategyInputDigest,
    CanonicalStrategyOutputArtifact, CommitStrategyDescriptor, CommitStrategyFamilyName,
    CommitStrategyId, CommitStrategySemanticName, CommitStrategyVersion, PersistentArtifactName,
    StrategyCallerProvenance, StrategyExecutionDraft, StrategyExecutionResult,
    StrategyExecutionSummary, StrategyInputSchemaName, StrategyInputSchemaVersion,
    StrategyIntentName, StrategyLoweringSummary, StrategyMutationProgram, StrategyOutputSchemaName,
    StrategyPacketContract, StrategyPreviewValidationCostSummary, StrategyReadContract,
    StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
    StrategyRequestOrigin, StrategyTraversalBasis,
};
use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::runtime::builder::RelationalRuntimeBuilder;
use crate::symbols::data::ClientKey;
use crate::transactions::data::{AspectFieldPatch, CommitValidationSummary, EntitySpec};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};

fn descriptor() -> CommitStrategyDescriptor {
    CommitStrategyDescriptor::new(
        CommitStrategyId(41),
        CommitStrategySemanticName::new("strategy.intent.reconcile"),
        CommitStrategyFamilyName::new("strategy.intent"),
        CommitStrategyVersion::new(1, 0),
        StrategyIntentName::new("reconcile.desired.state"),
        StrategyInputSchemaName::new("intent.reconcile.input.v1"),
        StrategyInputSchemaVersion(1),
        StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
        StrategyReadContract {
            scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
            locality_class: StrategyReadLocalityClass::SinglePartition,
            traversal_basis: StrategyTraversalBasis::NoTraversal,
            packet_contract: StrategyPacketContract::ProjectionOnly,
            cost_class: StrategyReadCostClass::ORequestedSurface,
        },
        PersistentArtifactName::new("strategy.intent.reconcile"),
    )
}

fn canonical_request() -> CanonicalStrategyCommitRequest {
    let descriptor = descriptor();
    CanonicalStrategyCommitRequest::new(
        CommitStrategyId(41),
        descriptor.digest(),
        CanonicalStrategyInputArtifact::new(
            StrategyInputSchemaName::new("intent.reconcile.input.v1"),
            StrategyInputSchemaVersion(1),
            b"replicas=3".to_vec().into(),
            CanonicalStrategyInputDigest([9; 32]),
            PersistentArtifactName::new("strategy.intent.reconcile.input"),
        ),
        StrategyCallerProvenance {
            request_origin: StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        },
    )
}

fn execution_draft(request: &CanonicalStrategyCommitRequest) -> StrategyExecutionDraft {
    let batch = WorkerIntentBatch::new("reconcile-deployment").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId(1),
            kind_id: KindId(1),
            client_key: ClientKey::from("deployment-a"),
            fields: AspectFieldPatch::from_locator(
                crate::transactions::data::planned_single_field_locator(
                    AspectKey::new("name").expect("valid name aspect key"),
                    FieldKey::new("name").expect("valid name field key"),
                ),
                AspectValue::String(InternedString::Raw("deployment-a".to_string())),
            ),
        }),
    ));

    StrategyExecutionDraft::from_measured_result(
        request,
        StrategyExecutionResult::new(
            CanonicalStrategyOutputArtifact::new(
                StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
                b"status=planned".to_vec(),
                PersistentArtifactName::new("strategy.intent.reconcile.output"),
            ),
            StrategyMutationProgram::new(vec![batch]),
        ),
        StrategyExecutionSummary::default(),
    )
}

fn lowered_bundle() -> StrategyCommitArtifactBundle {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(crate::tests::support::test_schema_registry())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let (transaction_validation_input, mut authority) =
        crate::tests::support::test_owner_strategy_authority(&runtime, None);
    let lowered = authority
        .lower_execution_with_input(
            &worth_execution::ExecutionRequest::serial(
                &crate::test_execution_policy::serial_request(),
            ),
            &runtime,
            &request,
            &execution,
            transaction_validation_input,
        )
        .expect("lowered strategy plan");

    StrategyCommitArtifactBundle::from_lowered(&lowered, &descriptor(), runtime.runtime_config())
}

#[test]
fn strategy_commit_artifact_bundle_carries_consistent_typed_artifacts() {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(crate::tests::support::test_schema_registry())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let (transaction_validation_input, mut authority) =
        crate::tests::support::test_owner_strategy_authority(&runtime, None);
    let lowered = authority
        .lower_execution_with_input(
            &worth_execution::ExecutionRequest::serial(
                &crate::test_execution_policy::serial_request(),
            ),
            &runtime,
            &request,
            &execution,
            transaction_validation_input,
        )
        .expect("lowered strategy plan");
    let bundle = StrategyCommitArtifactBundle::from_lowered(
        &lowered,
        &descriptor(),
        runtime.runtime_config(),
    );

    bundle.validate_consistency().expect("consistent bundle");
    assert_eq!(
        bundle.merge_descriptor().semantic_name().as_str(),
        "strategy.intent.reconcile"
    );
    assert_eq!(
        bundle.replay_request().canonical_input().canonical_bytes(),
        request.canonical_input().canonical_bytes()
    );
    assert_eq!(
        bundle
            .replay_descriptor()
            .runtime_determinism_basis()
            .schema_registry_digest(),
        &runtime.schema_registry().authority_digest_bytes()
    );
}

#[test]
fn strategy_commit_artifact_bundle_rejects_drift_between_summary_and_descriptor() {
    let bundle = lowered_bundle();
    let mut drifted_bundle = bundle.clone();
    let summary = bundle.lowering_summary();
    drifted_bundle.lowering_summary = StrategyLoweringSummary::new(
        99,
        summary.total_intent_count(),
        summary.touched_partition_count(),
        summary.cross_partition_relation_count(),
        summary.normalized_client_key_count(),
        summary.lineage_transition_count(),
        summary.projected_entity_record_reads(),
        summary.projected_relation_record_reads(),
        summary.projected_partition_reads(),
    );

    let error = drifted_bundle.validate_consistency().unwrap_err();
    assert_eq!(
        error,
        "strategy lowering summary does not match strategy replay descriptor digest"
    );
}

#[test]
fn strategy_commit_artifact_bundle_rejects_preview_validation_cost_drift() {
    let bundle = lowered_bundle().with_preview_validation(
        CommitValidationSummary {
            execution_count: 3,
            ..CommitValidationSummary::default()
        },
        StrategyPreviewValidationCostSummary::new(
            crate::identity::data::VersionId(1),
            1,
            1,
            1,
            0,
            2,
        ),
        None,
        crate::identity::data::VersionId(0),
    );
    let mut drifted_bundle = bundle.clone();
    drifted_bundle.preview_validation_cost = Some(StrategyPreviewValidationCostSummary::new(
        crate::identity::data::VersionId(1),
        1,
        1,
        1,
        0,
        3,
    ));

    let error = drifted_bundle.validate_consistency().unwrap_err();
    assert_eq!(
        error,
        "strategy preview validation cost does not match strategy replay descriptor digest"
    );
}

#[test]
fn strategy_intent_scope_targets_preserve_aspect_identity() {
    let field = FieldKey::new("replicas").expect("valid field");
    let desired_target = crate::transactions::data::planned_single_field_locator(
        AspectKey::new("deployment.desired").expect("valid desired aspect"),
        field.clone(),
    );
    let observed_target = crate::transactions::data::planned_single_field_locator(
        AspectKey::new("deployment.observed").expect("valid observed aspect"),
        field,
    );

    assert_ne!(
        native_entity_fields_scope_digest(EntityId::new(PartitionId(1), 7, 0), &[desired_target]),
        native_entity_fields_scope_digest(EntityId::new(PartitionId(1), 7, 0), &[observed_target]),
        "strategy scope digest must not collapse same field path under different aspects"
    );
}

#[test]
fn strategy_merge_descriptor_carries_typed_intent_scope_targets() {
    let field = FieldKey::new("replicas").expect("valid field");
    let target = crate::transactions::data::planned_single_field_locator(
        AspectKey::new("deployment.desired").expect("valid aspect"),
        field,
    );
    let descriptor = StrategyMergeDescriptor {
        strategy_id: CommitStrategyId(41),
        descriptor_digest: descriptor().digest(),
        semantic_name: CommitStrategySemanticName::new("strategy.intent.reconcile"),
        family_name: CommitStrategyFamilyName::new("strategy.intent"),
        version: CommitStrategyVersion::new(1, 0),
        intent_name: StrategyIntentName::new("reconcile.desired.state"),
        intent_scope_digest: StrategyIntentScopeDigest::new([5; 32]),
        intent_scope_targets: Arc::from([target.clone()]),
        merge_semantics: StrategyMergeSemantics::new(
            StrategyMergeConflictClass::IntentReconciliation,
            true,
            true,
        ),
        lowering_summary_digest: [9; 32],
    };

    assert_eq!(descriptor.intent_scope_targets(), &[target]);
}
