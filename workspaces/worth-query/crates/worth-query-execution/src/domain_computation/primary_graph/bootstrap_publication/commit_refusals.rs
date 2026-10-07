use super::execution_refusals::{authority, isolated, request};
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryManagedComputationResourceDenial as Resource,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as QueryKind,
};
use std::{num::NonZeroUsize, time::Instant};
use worth_execution::{CancellationSource, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};

pub(in crate::domain_computation::primary_graph) fn refused_commit(
    request: LeaseRequest,
) -> worth_relational::facade::transactions::TransactionCommitError {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "bootstrap.entity".to_owned(),
            schema_id: SchemaId("bootstrap-commit-probe".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
        })
        .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("bootstrap-commit").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("bootstrap-commit"),
                    fields: AspectFieldPatch::default(),
                }),
            )),
        )
        .unwrap();
    let lease = authority().request_lease(request).unwrap();
    // Bootstrap has no production lease injector until 7.6. Exercise its
    // error-mapping seam using a real owner refusal, without threading a lease.
    runtime
        .prepare_branch_transaction_with_lease(transaction, &lease)
        .expect_err("commit preparation refused")
}

#[test]
fn bootstrap_commit_preserves_real_leased_work_refusal() {
    isolated(
        concat!(
            module_path!(),
            "::bootstrap_commit_preserves_real_leased_work_refusal"
        ),
        || {
            let mut request = request();
            request.policy = ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024, 0),
            );
            let error = refused_commit(request);
            let worth_relational::facade::transactions::TransactionCommitError::Execution {
                denial,
                ..
            } = &error
            else {
                panic!("real execution refusal required");
            };
            assert_eq!(
                denial.kind,
                worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(
                    Cause::WorkExhausted
                )
            );
            let partition_identity = denial.partition_identity;
            assert_eq!(
                partition_identity,
                Some(1),
                "the entity-kind packet boundary survives"
            );
            let validation = crate::domain_computation::primary_graph::provider::invariant_execution_failure::map_validation_failure(error.clone());
            assert_eq!(
                validation.kind(),
                crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ExecutionDenied(
                    QueryKind::ExecutionResource {
                        denial: Resource::WorkExhausted,
                        partition_identity,
                        policy_ancestor: None
                    }
                )
            );
            let denied = map_bootstrap_commit_denial(error);
            assert_eq!(
                denied.kind(),
                WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                    kind: QueryKind::ExecutionResource {
                        denial: Resource::WorkExhausted,
                        partition_identity,
                        policy_ancestor: None
                    },
                }
            );
            let WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied { kind } =
                denied.kind()
            else {
                panic!("installation lost the execution refusal");
            };
            let application = crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(
            crate::domain_computation::WorthQueryProviderSessionFailure::new(
                kind, crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "commit preparation execution refused", Default::default(),
            ),
        );
            let observed = WorthQueryApplicationCommitOutcome::Denied(application)
                .require_committed()
                .expect_err("the receipt requirement preserves a refusal");
            let WorthQueryApplicationCommitOutcome::Denied(observed) = observed else {
                panic!("the receipt requirement folded the refusal");
            };
            assert_eq!(
                observed.kind(),
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::ExecutionResource {
                    denial: Resource::WorkExhausted,
                    partition_identity,
                    policy_ancestor: None,
                }
            );
        },
    );
}

#[test]
fn bootstrap_commit_preserves_real_control_stops() {
    isolated(
        concat!(
            module_path!(),
            "::bootstrap_commit_preserves_real_control_stops"
        ),
        || {
            let cancellation = CancellationSource::new();
            cancellation.cancel();
            let mut canceled = request();
            canceled.cancellation = cancellation.token();
            let mut expired = request();
            expired.deadline = Some(Instant::now());
            for (request, expected) in
                [(canceled, Control::Cancelled), (expired, Control::TimedOut)]
            {
                let observed = map_bootstrap_commit_denial(refused_commit(request));
                assert_eq!(
                    observed.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                        kind: expected
                    }
                );
            }
        },
    );
}

#[test]
fn bootstrap_operation_interruption_keeps_heads_generic_commit_rejection() {
    use worth_relational::facade::mvcc::{
        RelationalCancellationSource, RelationalInterruptionBoundary, RelationalOperationControl,
    };
    use worth_relational::facade::transactions::TransactionCommitError;
    let source = RelationalCancellationSource::new();
    source.cancel();
    let canceled: RelationalOperationControl = source.token().into();
    let expired = RelationalOperationControl::uninterrupted().with_deadline(Instant::now());
    for control in [canceled, expired] {
        let event = control
            .observe(RelationalInterruptionBoundary::CandidatePreparation)
            .unwrap();
        let observed = map_bootstrap_commit_denial(TransactionCommitError::interrupted(event));
        assert_eq!(
            observed.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
        );
    }
}

pub(in crate::domain_computation::primary_graph) fn work_refusal(
) -> worth_relational::facade::transactions::TransactionCommitError {
    let mut request = request();
    request.policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024, 0),
    );
    refused_commit(request)
}
