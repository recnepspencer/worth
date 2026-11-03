use worth_foundational::facade::AspectValue;
use worth_foundational::facade::BoundaryProtocolIdentity;
use worth_foundational::facade::BoundaryProtocolVersion;
use worth_foundational::facade::InternedString;
use worth_query_declaration::facade::application_schema::ApplicationExternalEffectProtocol;
use worth_query_installation::facade::InstalledExternalEffectContract;
use worth_relational::facade::mvcc::BranchBoundRelationalTransaction;
use worth_relational::facade::transactions::CommitResult;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::correlation::derive_external_effect_correlation_identity;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::outbox::dispatch_outbox_create_intent;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::correlation::ExternalEffectCorrelationBasis;
use worth_query_execution::facade::domain_computation::WorthQueryDispatchOutboxRecord;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_branch::primary_relational_branch_id;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBinding;

pub fn commit_record(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    identity: u64,
) -> (
    WorthQueryCommittedDispatchOutboxBinding,
    worth_relational::facade::history::RelationalCommitReceipt,
    u64,
) {
    let record = record_for(identity);
    let branch = primary_relational_branch_id();
    let (binding, commit, runtime_id) = provider.graph.with_runtime_mut(request_execution, |runtime| {
        let mut transaction: BranchBoundRelationalTransaction = {
    let transaction_validation_input = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .expect("main branch binding");
    runtime
        .begin_branch_transaction(
            &transaction_validation_input,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context")
};
        transaction.push_batch(
            WorkerIntentBatch::new("committed-outbox-owner-test").push(
                dispatch_outbox_create_intent(
                    Some(provider.graph.layout.provider_dispatch_outbox()),
                    Some(&record),
                )
                .unwrap(),
            ),
         worth_execution::ExecutionAllocationPolicy::SystemAllocation).expect("test staging stays within configured resource budgets");
        let committed = transaction.commit(request_execution, runtime).unwrap();
        let binding = WorthQueryCommittedDispatchOutboxBinding::fixture_from_commit(
            provider.graph.layout.provider_dispatch_outbox(),
            Some(&record),
            &committed,
        )
        .unwrap()
        .unwrap();
        let commit = committed.outcome().commit.clone();
        retain_commit_basis(provider, runtime, &committed);
        let snapshot = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::exact_basis_access::open_current_branch_snapshot(runtime, &branch)
            .unwrap();
        let runtime_id = snapshot.runtime_instance_id();
        release_commit_snapshot(runtime, &committed);
        worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        (binding, commit, runtime_id)
    });
    (binding, commit, runtime_id)
}

pub fn release_commit_snapshot(
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    committed: &CommitResult,
) {
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
        runtime,
        &committed.snapshot,
    );
}

pub fn retain_commit_basis(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    committed: &CommitResult,
) {
    let branch_identity = runtime
        .branch_identity(&committed.commit.branch_id)
        .expect("committed fixture branch remains owner-issued");
    let (_, basis) = runtime
        .observe_branch(&branch_identity)
        .expect("committed fixture basis remains owner-observable");

    let retention = runtime
        .retain_component_basis(&basis)
        .expect("the committed fixture's native basis remains available");
    provider
        .receipt_basis_retention
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .retain(committed.commit.commit_id, retention, false);
}

pub fn record_for(identity: u64) -> WorthQueryDispatchOutboxRecord {
    let correlation = derive_external_effect_correlation_identity(ExternalEffectCorrelationBasis {
        correlation_family:
            worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                "owner-test",
            )
            .unwrap(),
        operation_slot: "notify",
        operation_version: 1,
        outcome_identity: identity,
        idempotency_key: &[identity as u8; 32],
        branch: "main",
    })
    .unwrap();
    WorthQueryDispatchOutboxRecord::from_installed_contract(
        correlation,
        &InstalledExternalEffectContract::Declared {
            correlation_family:
                worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                    "owner-test",
                )
                .unwrap(),
            effect: "OwnerTestEffect".to_owned(),
            rust_payload_type: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "worth.query.test.owner-payload.v1",
            ),
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::new("test.owner.payload"),
                BoundaryProtocolVersion::new(1),
            ),
            maximum_payload_bytes: 24,
            inbound: None,
        },
        vec![identity as u8; 8],
        identity,
    )
    .unwrap()
}

pub fn string(value: String) -> AspectValue {
    AspectValue::String(InternedString::from(value))
}
