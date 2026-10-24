pub(super) use super::super::CommitStrategiesAuthorityFacade;
pub(super) use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact, CanonicalStrategyInputDigest,
    CanonicalStrategyOutputArtifact, CommitStrategyDescriptor, CommitStrategyExecutionRegistration,
    CommitStrategyExecutor, CommitStrategyFamilyName, CommitStrategyId, CommitStrategyRegistration,
    CommitStrategySemanticName, CommitStrategyVersion, PersistentArtifactName,
    StrategyCallerProvenance, StrategyExecutionDraft, StrategyExecutionResult,
    StrategyExecutionSummary, StrategyExecutorFailure, StrategyInputSchemaName,
    StrategyInputSchemaVersion, StrategyIntentName, StrategyMutationProgram,
    StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract, StrategyReadCostClass,
    StrategyReadLocalityClass, StrategyReadScopeClass, StrategyRequestOrigin,
    StrategyTraversalBasis,
};
pub(super) use crate::commit_strategies::strategies::{
    AspectFieldReconciliationInput, AspectFieldReconciliationStrategy,
    EntityReplacementReconciliationInput, EntityReplacementReconciliationStrategy,
    IntentReconciliationInput, IntentReconciliationStrategy, ReplicaConvergenceInput,
    ReplicaConvergenceStrategy,
};
pub(super) use crate::facade::history::BranchId;
pub(super) use crate::facade::merge::{MergeIntent, MergePlanningRequest};
pub(super) use crate::facade::replay::{
    RelationalReplayRequest, ReplayExecutionMode, ReplayFailureClass, ReplayMismatchClass,
    ReplayObservableSurface, ReplayVerificationMode,
};
pub(super) use crate::facade::transactions::{
    CreateIntent, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
pub(super) use crate::identity::data::{EntityId, KindId, PartitionId};
pub(super) use crate::runtime::builder::RelationalRuntimeBuilder;
pub(super) use crate::snapshots::data::SnapshotHandle;
pub(super) use crate::symbols::data::ClientKey;
pub(super) use crate::tests::support::{
    changed_entities, entity_field_aspect, entity_u64_field_aspect, lifecycle_aspect,
    read_entity_name, AspectSchemaFixture,
};
pub(super) use crate::transactions::data::AspectFieldPatch;
pub(super) use crate::transactions::data::{EntitySpec, TransactionCommitError};
pub(super) use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, InternedString,
    LocatorAuthority,
};

pub(super) fn strategy_schema_registry() -> crate::schema::data::RelationalSchemaRegistry {
    AspectSchemaFixture {
        entity_aspects: vec![
            entity_field_aspect(
                crate::tests::support::aspect_key("name"),
                crate::tests::support::field_key("name"),
            ),
            entity_u64_field_aspect(
                crate::tests::support::aspect_key("replicas"),
                crate::tests::support::field_key("replicas"),
            ),
            lifecycle_aspect(),
        ],
        ..AspectSchemaFixture::default()
    }
    .build_registry()
}

pub(super) fn strategy_name_and_replicas_patch(name: &str, replicas: u64) -> AspectFieldPatch {
    AspectFieldPatch::from(std::collections::BTreeMap::from([
        (
            crate::transactions::data::planned_single_field_locator(
                AspectKey::new("name").expect("valid name aspect key"),
                FieldKey::new("name").expect("valid name field key"),
            ),
            AspectValue::String(InternedString::Raw(name.to_string())),
        ),
        (
            crate::transactions::data::planned_single_field_locator(
                AspectKey::new("replicas").expect("valid replicas aspect key"),
                FieldKey::new("replicas").expect("valid replicas field key"),
            ),
            AspectValue::UInt64(replicas),
        ),
    ]))
}

pub(super) fn strategy_field_locator(
    aspect_key: AspectKey,
    field_key: FieldKey,
) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key,
        CanonicalFieldPath::single(field_key),
    )
}

pub(super) fn strategy_descriptor() -> CommitStrategyDescriptor {
    strategy_descriptor_named(
        CommitStrategyId(41),
        "strategy.intent.reconcile",
        "strategy.intent",
        "reconcile.desired.state",
    )
}

pub(super) fn strategy_descriptor_named(
    id: CommitStrategyId,
    semantic_name: &str,
    family_name: &str,
    intent_name: &str,
) -> CommitStrategyDescriptor {
    CommitStrategyDescriptor::new(
        id,
        CommitStrategySemanticName::new(semantic_name),
        CommitStrategyFamilyName::new(family_name),
        CommitStrategyVersion::new(1, 0),
        StrategyIntentName::new(intent_name),
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
        PersistentArtifactName::new(semantic_name),
    )
}

pub(super) fn strategy_registration() -> CommitStrategyRegistration {
    CommitStrategyRegistration::new(strategy_descriptor()).expect("valid strategy registration")
}

#[derive(Clone, Copy)]
pub(super) struct PlanningExecutor;

impl CommitStrategyExecutor for PlanningExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &CanonicalStrategyCommitRequest,
        _observation: &crate::commit_strategies::data::StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, StrategyExecutorFailure> {
        Ok(execution_result(request))
    }
}

#[derive(Clone, Copy)]
pub(super) struct DeterministicFailureExecutor;

impl CommitStrategyExecutor for DeterministicFailureExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _request: &CanonicalStrategyCommitRequest,
        _observation: &crate::commit_strategies::data::StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, StrategyExecutorFailure> {
        Err(StrategyExecutorFailure::new(
            crate::commit_strategies::data::StrategyExecutorFailureClass::DomainRejection,
            "deterministic hostile replay failure",
        ))
    }
}

pub(crate) fn persisted_intent_runtime(
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
    include_executor: bool,
) -> crate::facade::runtime::RelationalRuntime {
    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(161));
    let mut builder = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .durable_medium(medium)
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone())
                .expect("intent strategy registration"),
        );
    if include_executor {
        builder = builder.commit_strategy_executor(
            IntentReconciliationStrategy::execution_registration(&descriptor),
        );
    }
    builder.build()
}

pub(super) fn persisted_intent_runtime_with_failing_executor(
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> crate::facade::runtime::RelationalRuntime {
    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(161));
    RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .durable_medium(medium)
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone())
                .expect("intent strategy registration"),
        )
        .commit_strategy_executor(CommitStrategyExecutionRegistration::new(
            &descriptor,
            DeterministicFailureExecutor,
        ))
        .build()
}

pub(crate) fn execute_persisted_intent_strategy_commit(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    entity: EntityId,
) -> crate::facade::transactions::CommitResult {
    let request = runtime
        .commit_strategies()
        .canonicalize_request(
            &IntentReconciliationInput {
                entity_id: entity,
                desired_aspect_fields: crate::transactions::data::AspectFieldPatch::from_locator(
                    crate::transactions::data::planned_single_field_locator(
                        worth_foundational::facade::AspectKey::new("name")
                            .expect("valid test aspect key"),
                        FieldKey::new("name").expect("valid test field key"),
                    ),
                    worth_foundational::facade::AspectValue::String(
                        worth_foundational::facade::InternedString::Raw("after".to_string()),
                    ),
                ),
            }
            .into_native_canonical_request(StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            })
            .expect("native canonical strategy request"),
        )
        .expect("canonical request");
    let snapshot = runtime.visibility_authority().snapshot();
    let execution = runtime
        .commit_strategies()
        .execute(&request, &snapshot, request_execution)
        .expect("strategy execution");
    let (transaction_validation_input, mut authority) =
        crate::tests::support::test_owner_strategy_authority(runtime, None);
    let lowered = authority
        .lower_execution_with_input(runtime, &request, &execution, transaction_validation_input)
        .expect("lowered strategy plan");
    let validated = authority
        .validate_lowered_plan(request_execution, runtime, lowered)
        .expect("validated strategy plan");
    authority
        .execute_validated_commit(request_execution, runtime, validated)
        .expect("validated strategy commit")
}

pub(super) fn canonical_request() -> CanonicalStrategyCommitRequest {
    let descriptor = strategy_descriptor();
    CanonicalStrategyCommitRequest::new(
        CommitStrategyId(41),
        descriptor.digest(),
        CanonicalStrategyInputArtifact::new(
            StrategyInputSchemaName::new("intent.reconcile.input.v1"),
            StrategyInputSchemaVersion(1),
            b"fixture-input".to_vec().into(),
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

pub(super) fn execution_result(
    _request: &CanonicalStrategyCommitRequest,
) -> StrategyExecutionResult {
    let batch = WorkerIntentBatch::new("reconcile-deployment").push(MutationIntent::Create(
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId(1),
            kind_id: KindId(1),
            client_key: ClientKey::from("deployment-a"),
            fields: strategy_name_and_replicas_patch("deployment-a", 3),
        }),
    ));

    StrategyExecutionResult::new(
        CanonicalStrategyOutputArtifact::new(
            StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
            b"status=planned".to_vec(),
            PersistentArtifactName::new("strategy.intent.reconcile.output"),
        ),
        StrategyMutationProgram::new(vec![batch]),
    )
}

pub(super) fn update_execution_draft(
    request: &CanonicalStrategyCommitRequest,
    entity_id: EntityId,
    name: &str,
) -> StrategyExecutionDraft {
    let batch = WorkerIntentBatch::new("reconcile-update").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id,
            fields: crate::transactions::data::AspectFieldPatch::from_locator(
                crate::transactions::data::planned_single_field_locator(
                    AspectKey::new("name").expect("valid name aspect key"),
                    FieldKey::new("name").expect("valid name field key"),
                ),
                AspectValue::String(InternedString::Raw(name.to_string())),
            ),
        }),
    ));

    StrategyExecutionDraft::from_measured_result(
        request,
        StrategyExecutionResult::new(
            CanonicalStrategyOutputArtifact::new(
                StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
                format!("status=planned;name={name}").into_bytes(),
                PersistentArtifactName::new("strategy.intent.reconcile.output"),
            ),
            StrategyMutationProgram::new(vec![batch]),
        ),
        StrategyExecutionSummary::default(),
    )
}

pub(super) fn execution_draft(request: &CanonicalStrategyCommitRequest) -> StrategyExecutionDraft {
    StrategyExecutionDraft::from_measured_result(
        request,
        execution_result(request),
        StrategyExecutionSummary::default(),
    )
}

pub(crate) fn lowered_commit_fixture() -> (
    crate::runtime::RelationalRuntime,
    crate::commit_strategies::data::LoweredStrategyCommitPlan,
) {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let request = canonical_request();
    let draft = execution_draft(&request);
    let (input, mut authority) =
        crate::tests::support::test_owner_strategy_authority(&runtime, None);
    let lowered = authority
        .lower_execution_with_input(&runtime, &request, &draft, input)
        .unwrap();
    (runtime, lowered)
}
