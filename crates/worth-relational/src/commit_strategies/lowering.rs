use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, LoweredStrategyCommitPlan, StrategyExecutionDraft,
    StrategyLoweringError, StrategyLoweringProvenance, StrategyLoweringSummary,
};
use crate::runtime::RelationalRuntime;

pub(crate) fn lower_execution(
    runtime: &RelationalRuntime,
    request: &CanonicalStrategyCommitRequest,
    execution: &StrategyExecutionDraft,
    mut transaction: crate::mvcc::BranchBoundRelationalTransaction,
) -> Result<LoweredStrategyCommitPlan, StrategyLoweringError> {
    validate_execution_binding(request, execution)?;
    transaction
        .ensure_current_basis_for_runtime(runtime)
        .map_err(StrategyLoweringError::mutation_conflict)?;
    let preparation = runtime.preparation_runtime_snapshot();
    let selected_branch_state = runtime
        .selected_branch_state(transaction.basis())
        .map_err(StrategyLoweringError::preparation)?;
    for worker_batch in execution
        .mutation_program()
        .worker_batches()
        .iter()
        .cloned()
    {
        transaction
            .push_batch(
                worker_batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(|denial| StrategyLoweringError::mutation_conflict(denial.into_conflict()))?;
    }
    runtime
        .services
        .symbols
        .with_read(|symbols| {
            transaction.validate_staged_branch_locality(selected_branch_state.state(), symbols)
        })
        .map_err(StrategyLoweringError::mutation_conflict)?;

    let bulk_mutation_batch = transaction
        .admit_provenance_complete_bulk_mutation_batch(runtime)
        .map_err(StrategyLoweringError::mutation_conflict)?;
    let intents = transaction
        .normalized_intents_for_merge(
            &preparation,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(StrategyLoweringError::mutation_conflict)?;
    let merged_plan = transaction
        .build_merged_plan_for_state(&preparation, selected_branch_state.state(), intents)
        .map_err(StrategyLoweringError::mutation_conflict)?;
    transaction
        .footprint
        .derive_validation_dependencies(
            &merged_plan,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(|denial| StrategyLoweringError::mutation_conflict(denial.into_conflict()))?;
    let lowering_provenance =
        StrategyLoweringProvenance::from_request_and_execution(request, execution);
    let lowering_summary = build_lowering_summary(execution, bulk_mutation_batch.as_ref());

    Ok(LoweredStrategyCommitPlan::new(
        request.clone(),
        execution.clone(),
        transaction,
        bulk_mutation_batch,
        selected_branch_state,
        merged_plan,
        lowering_provenance,
        lowering_summary,
    ))
}

pub(crate) fn strategy_transaction_admission_error(
    error: crate::mvcc::RelationalBranchTransactionAdmissionDenial,
) -> StrategyLoweringError {
    match error {
        crate::mvcc::RelationalBranchTransactionAdmissionDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => StrategyLoweringError::mutation_conflict(
            crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::ForeignRuntime {
                    expected_runtime_instance_id,
                    actual_runtime_instance_id,
                },
            ),
        ),
        crate::mvcc::RelationalBranchTransactionAdmissionDenial::BasisIdentityMismatch => {
            StrategyLoweringError::RequestExecutionMismatch {
                detail: "strategy transaction basis identity is inconsistent".to_owned(),
            }
        }
        denial => StrategyLoweringError::RequestExecutionMismatch {
            detail: format!("strategy transaction admission denied: {denial:?}"),
        },
    }
}

fn validate_execution_binding(
    request: &CanonicalStrategyCommitRequest,
    execution: &StrategyExecutionDraft,
) -> Result<(), StrategyLoweringError> {
    let binding = execution.request_binding();
    if binding.strategy_id() != request.strategy_id() {
        return Err(StrategyLoweringError::RequestExecutionMismatch {
            detail: format!(
                "execution draft strategy id {} does not match request strategy id {}",
                binding.strategy_id().0,
                request.strategy_id().0
            ),
        });
    }
    if binding.descriptor_digest() != request.descriptor_digest() {
        return Err(StrategyLoweringError::RequestExecutionMismatch {
            detail: "execution draft descriptor digest does not match canonical request descriptor digest"
                .to_string(),
        });
    }
    if binding.input_digest() != request.canonical_input().digest() {
        return Err(StrategyLoweringError::RequestExecutionMismatch {
            detail: "execution draft input digest does not match canonical request input digest"
                .to_string(),
        });
    }
    Ok(())
}

fn build_lowering_summary(
    execution: &StrategyExecutionDraft,
    bulk_mutation_batch: Option<&crate::transactions::data::ProvenanceCompleteBulkMutationBatch>,
) -> StrategyLoweringSummary {
    let planned = bulk_mutation_batch.map(|batch| batch.planned());
    StrategyLoweringSummary::new(
        execution.mutation_program().worker_batches().len(),
        execution.mutation_program().total_intent_count(),
        planned
            .map(|planned| planned.locality.touched_partitions.len())
            .unwrap_or(0),
        planned
            .map(|planned| planned.locality.cross_partition_relation_count)
            .unwrap_or(0),
        planned
            .map(|planned| planned.naming.normalized_client_keys.len())
            .unwrap_or(0),
        planned
            .map(|planned| planned.lineage.transitions.len())
            .unwrap_or(0),
        execution.summary().projected_entity_record_reads,
        execution.summary().projected_relation_record_reads,
        execution.summary().projected_partition_reads,
    )
}

#[cfg(test)]
#[path = "lowering_boundary_tests.rs"]
mod boundary_tests;

#[cfg(test)]
mod tests {
    use super::lower_execution;
    use crate::commit_strategies::data::{
        CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact,
        CanonicalStrategyInputDigest, CanonicalStrategyOutputArtifact,
        CommitStrategyDescriptorDigest, CommitStrategyId, PersistentArtifactName,
        StrategyCallerProvenance, StrategyExecutionDraft, StrategyExecutionResult,
        StrategyExecutionSummary, StrategyInputSchemaName, StrategyInputSchemaVersion,
        StrategyMutationProgram, StrategyOutputSchemaName, StrategyRequestOrigin,
    };
    use crate::facade::history::BranchId;
    use crate::facade::transactions::{CreateIntent, MutationIntent, WorkerIntentBatch};
    use crate::identity::data::{KindId, PartitionId};
    use crate::runtime::builder::RelationalRuntimeBuilder;
    use crate::symbols::data::ClientKey;
    use crate::transactions::data::{AspectFieldPatch, EntitySpec};
    use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};

    pub(super) fn canonical_request() -> CanonicalStrategyCommitRequest {
        CanonicalStrategyCommitRequest::new(
            CommitStrategyId(41),
            CommitStrategyDescriptorDigest([7; 32]),
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

    pub(super) fn execution_draft(
        request: &CanonicalStrategyCommitRequest,
    ) -> StrategyExecutionDraft {
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

        execution_draft_with_batches(request, vec![batch])
    }

    pub(super) fn execution_draft_with_batches(
        request: &CanonicalStrategyCommitRequest,
        batches: Vec<WorkerIntentBatch>,
    ) -> StrategyExecutionDraft {
        StrategyExecutionDraft::from_measured_result(
            request,
            StrategyExecutionResult::new(
                CanonicalStrategyOutputArtifact::new(
                    StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
                    b"status=planned".to_vec(),
                    PersistentArtifactName::new("strategy.intent.reconcile.output"),
                ),
                StrategyMutationProgram::new(batches),
            ),
            StrategyExecutionSummary::default(),
        )
    }

    #[test]
    fn lower_execution_routes_strategy_batches_through_transaction_admission() {
        let runtime = RelationalRuntimeBuilder::new()
            .schema_registry(crate::tests::support::test_schema_registry())
            .build();
        let request = canonical_request();
        let execution = execution_draft(&request);
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime);
        let transaction = runtime
            .begin_branch_transaction_with_owner_inputs(transaction_validation_input)
            .expect("owner context opens a branch-bound transaction");

        let lowered = lower_execution(&runtime, &request, &execution, transaction)
            .expect("lowered strategy plan");

        assert_eq!(lowered.request().strategy_id(), CommitStrategyId(41));
        assert_eq!(lowered.merged_plan().merged_intents.len(), 1);
        assert_eq!(
            lowered.execution().mutation_program().total_intent_count(),
            1
        );
        assert_eq!(
            lowered.lowering_provenance().mutation_program_digest(),
            execution.mutation_program().digest()
        );
        assert_eq!(lowered.lowering_summary().worker_batch_count(), 1);
        assert_eq!(lowered.lowering_summary().total_intent_count(), 1);
        assert_eq!(lowered.lowering_summary().normalized_client_key_count(), 1);
        assert_eq!(lowered.lowering_summary().lineage_transition_count(), 1);
        assert_eq!(lowered.lowering_summary().touched_partition_count(), 1);
        assert!(lowered.bulk_mutation_batch().is_some());
    }

    #[test]
    fn lower_execution_rejects_request_execution_mismatch() {
        let runtime = RelationalRuntimeBuilder::new().build();
        let request = canonical_request();
        let other_request = CanonicalStrategyCommitRequest::new(
            CommitStrategyId(42),
            request.descriptor_digest(),
            request.canonical_input().clone(),
            request.caller_provenance().clone(),
        );
        let execution = execution_draft(&request);
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime);
        let transaction = runtime
            .begin_branch_transaction_with_owner_inputs(transaction_validation_input)
            .expect("owner context opens a branch-bound transaction");

        let error = lower_execution(&runtime, &other_request, &execution, transaction).unwrap_err();

        assert!(matches!(
            error,
            crate::commit_strategies::data::StrategyLoweringError::RequestExecutionMismatch { .. }
        ));
    }

    #[test]
    fn transaction_admission_denies_missing_root_before_raw_key_normalization() {
        let runtime = crate::tests::support::runtime_with_test_schema();
        crate::tests::support::create_entity_outcome(&runtime, "strategy-root-source");
        let source = BranchId("main".to_owned());
        let (_, basis) = runtime
            .observe_fork_source(&source)
            .expect("committed main remains forkable");
        let child = BranchId("strategy-root-child".to_owned());
        runtime
            .fork_branch(child.clone(), basis)
            .expect("child branch installs a committed root");
        runtime
            .history
            .branch_cell_mut(&child)
            .expect("child remains registered")
            .clear_root_for_test();
        let symbols_before = runtime.services.symbols.clone();
        let symbol_table_before = runtime.config().identity.symbol_table.clone();
        let child_identity = runtime
            .branch_identity(&child)
            .expect("child identity remains owner-issued");
        let denial = runtime
            .admit_branch_basis(&child_identity)
            .expect_err("a committed branch without its exact root cannot mint authority");

        assert_eq!(
            denial,
            crate::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget
        );
        assert_eq!(runtime.services.symbols, symbols_before);
        assert_eq!(runtime.config().identity.symbol_table, symbol_table_before);
    }
}
