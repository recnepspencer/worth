use crate::internal_port::commit_strategies::data::execution::execution_draft::StrategyExecutionDraftOwnerPort as _;
use crate::internal_port::commit_strategies::data::lowering_meaning::StrategyLoweringProvenanceOwnerPort as _;
use crate::internal_port::commit_strategies::data::lowering_meaning::StrategyLoweringSummaryOwnerPort as _;
use crate::internal_port::commit_strategies::data::request::CanonicalStrategyCommitRequestOwnerPort as _;
use crate::internal_port::commit_strategies::data::request::CanonicalStrategyInputArtifactOwnerPort as _;
use std::sync::Arc;

use super::{
    StrategyCommitArtifactBundle, StrategyIntentScopeDigest, StrategyMergeConflictClass,
    StrategyMergeDescriptor, StrategyMergeSemantics,
};
use crate::commit_strategies::data::canonical_digest::native_entity_fields_scope_digest;
use crate::commit_strategies::data::descriptor::descriptor_record::CommitStrategyDescriptor;
use crate::commit_strategies::data::descriptor::read_contract::{
    CommitStrategyVersion, StrategyPacketContract, StrategyReadContract, StrategyReadCostClass,
    StrategyReadLocalityClass, StrategyReadScopeClass, StrategyTraversalBasis,
};
use crate::commit_strategies::data::descriptor::semantic_names::StrategyInputSchemaVersion;
use crate::commit_strategies::data::execution::execution_draft::{
    StrategyExecutionDraft, StrategyExecutionResult,
};
use crate::commit_strategies::data::execution::execution_summary::StrategyExecutionSummary;
use crate::commit_strategies::data::execution::mutation_program::StrategyMutationProgram;
use crate::commit_strategies::data::execution::output_artifact::CanonicalStrategyOutputArtifact;
use crate::commit_strategies::data::lowering_meaning::StrategyLoweringSummary;
use crate::commit_strategies::data::request::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact, CanonicalStrategyInputDigest,
    StrategyCallerProvenance, StrategyRequestOrigin,
};
use crate::commit_strategies::data::strategy_id::CommitStrategyId;
use crate::commit_strategies::data::validation::StrategyPreviewValidationCostSummary;
use crate::commit_strategies::data::{
    CommitStrategyFamilyName, CommitStrategySemanticName, PersistentArtifactName,
    StrategyInputSchemaName, StrategyIntentName, StrategyOutputSchemaName,
};
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::transactions::data::intents::mutation_intent::{CreateIntent, MutationIntent};
use crate::transactions::data::outcomes::commit_validation::CommitValidationSummary;
use crate::transactions::data::primitives::EntitySpec;
use crate::transactions::data::primitives::WorkerIntentBatch;
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
                crate::transactions::data::aspect_field_patch::planned_single_field_locator(
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

fn artifact_basis(
    request: CanonicalStrategyCommitRequest,
) -> crate::commit_strategies::data::artifact_basis::StrategyArtifactBasis {
    use crate::commit_strategies::data::lowering_meaning::StrategyLoweringProvenance;
    use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
    let execution = execution_draft(&request);
    let provenance = StrategyLoweringProvenance::from_request_and_execution(&request, &execution);
    let merged_plan = MergedCommitPlan {
        transaction_id: crate::transactions::data::primitives::TransactionId(1),
        merged_intents: execution
            .mutation_program()
            .worker_batches()
            .iter()
            .flat_map(|batch| batch.intents.iter().cloned())
            .collect(),
    };
    crate::commit_strategies::data::artifact_basis::StrategyArtifactBasis::new(
        request,
        execution,
        merged_plan,
        provenance,
        StrategyLoweringSummary::new(1, 1, 1, 0, 1, 0, 0, 0, 0),
    )
}

fn lowered_bundle() -> StrategyCommitArtifactBundle {
    StrategyCommitArtifactBundle::from_lowered(
        &artifact_basis(canonical_request()),
        &descriptor(),
        &crate::config::data::RelationalRuntimeConfig::default(),
    )
}

#[test]
fn strategy_commit_artifact_bundle_carries_consistent_typed_artifacts() {
    let config = crate::config::data::RelationalRuntimeConfig::default();
    let request = canonical_request();
    let bundle = StrategyCommitArtifactBundle::from_lowered(
        &artifact_basis(request.clone()),
        &descriptor(),
        &config,
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
        &config.schema.registry.authority_digest_bytes()
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
    let desired_target =
        crate::transactions::data::aspect_field_patch::planned_single_field_locator(
            AspectKey::new("deployment.desired").expect("valid desired aspect"),
            field.clone(),
        );
    let observed_target =
        crate::transactions::data::aspect_field_patch::planned_single_field_locator(
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
    let target = crate::transactions::data::aspect_field_patch::planned_single_field_locator(
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
        intent_scope_targets_memory: None,
        merge_semantics: StrategyMergeSemantics::new(
            StrategyMergeConflictClass::IntentReconciliation,
            true,
            true,
        ),
        lowering_summary_digest: worth_foundational::CanonicalDigestId::new([9; 32]),
    };

    assert_eq!(descriptor.intent_scope_targets(), &[target]);
}
