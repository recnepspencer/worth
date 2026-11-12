//! Owner-local proofs for commit-to-receipt outbox resolution.

use worth_foundational::facade::BoundaryProtocolIdentity;
use worth_foundational::facade::BoundaryProtocolVersion;
use worth_query_declaration::facade::application_schema::ApplicationExternalEffectProtocol;
use worth_query_installation::facade::InstalledExternalEffectContract;
use worth_relational::facade::transactions::RecordRef;
use worth_relational::facade::transactions::WorkerIntentBatch;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxBindingDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::session_commit::outbox_commit::WorthQueryCommittedDispatchOutboxResolution;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::outbox::bind_dispatch_outbox_create_intent;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::correlation::derive_external_effect_correlation_identity;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::correlation::ExternalEffectCorrelationBasis;
use worth_query_execution::facade::domain_computation::WorthQueryDispatchOutboxRecord;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::outbox::WorthQueryPendingDispatchOutbox;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn real_commit_mapping_denial_reaches_receipt_projection() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let evidence = commit_only_other_outbox(request_execution);
    let seal = evidence
        .committed_resolution
        .seal_for_receipt_for_test()
        .expect("the committed create resolves");
    let binding = seal
        .binding()
        .expect("the committed create has one binding");
    assert_eq!(
        binding.record_for_test(),
        evidence.committed_pending.record_for_test()
    );
    assert_eq!(binding.record_ref_for_test(), &evidence.expected_record_ref);

    assert_eq!(
        evidence
            .requested_resolution
            .seal_for_receipt_for_test()
            .map(|seal| seal.into_binding()),
        Err(WorthQueryCommittedDispatchOutboxBindingDenial::CreatedEntityMissing)
    );
}

#[test]
fn real_commit_with_no_requested_outbox_projects_honest_absence() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let evidence = commit_only_other_outbox(request_execution);
    assert_eq!(
        evidence
            .absent_resolution
            .seal_for_receipt_for_test()
            .expect("honest absence is a successful receipt projection")
            .into_binding(),
        None
    );
}

pub struct CommittedOutboxResolutionEvidence {
    pub committed_pending: WorthQueryPendingDispatchOutbox,
    pub committed_resolution: WorthQueryCommittedDispatchOutboxResolution,
    pub requested_resolution: WorthQueryCommittedDispatchOutboxResolution,
    pub absent_resolution: WorthQueryCommittedDispatchOutboxResolution,
    pub expected_record_ref: RecordRef,
}

pub fn commit_only_other_outbox(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CommittedOutboxResolutionEvidence {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let provider = (*world.application.runtime_for_test()).primary_provider_for_test();
    let (_, requested) = bind_dispatch_outbox_create_intent(
        Some((*(*provider.graph_for_test()).layout_for_test()).provider_dispatch_outbox_for_test()),
        Some(&record(1)),
        worth_relational::facade::identity::PartitionId::main(),
    )
    .expect("requested outbox create binds");
    let (committed_intent, committed_pending) = bind_dispatch_outbox_create_intent(
        Some((*(*provider.graph_for_test()).layout_for_test()).provider_dispatch_outbox_for_test()),
        Some(&record(2)),
        worth_relational::facade::identity::PartitionId::main(),
    )
    .expect("committed outbox create binds");
    (*provider
        .graph_for_test())
        .with_runtime_for_test(request_execution, |runtime| {
            let mut transaction = {
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
            transaction
                .push_batch(
                    WorkerIntentBatch::new("outbox-resolution-owner-proof").push(committed_intent),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("test staging stays within configured resource budgets");
            let committed = transaction
                .commit(request_execution, runtime)
                .expect("the other outbox commits");
            let expected_record_ref = RecordRef::Entity(
                committed
                    .created_entity(committed_pending.created_entity_for_test())
                    .expect("the exact committed create has an owner-minted mapping"),
            );
            let evidence = CommittedOutboxResolutionEvidence {
                committed_resolution: WorthQueryCommittedDispatchOutboxResolution::from_commit_for_test(
                    Some(&committed_pending),
                    &committed,
                ),
                requested_resolution: WorthQueryCommittedDispatchOutboxResolution::from_commit_for_test(
                    Some(&requested),
                    &committed,
                ),
                absent_resolution: WorthQueryCommittedDispatchOutboxResolution::from_commit_for_test(
                    None, &committed,
                ),
                committed_pending,
                expected_record_ref,
            };
            worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
                runtime,
                &committed.snapshot,
            );
            evidence
        })
}

pub fn record(identity: u64) -> WorthQueryDispatchOutboxRecord {
    let correlation = derive_external_effect_correlation_identity(ExternalEffectCorrelationBasis {
        correlation_family:
            worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                "receipt-resolution-test",
            )
            .unwrap(),
        operation_slot: "notify",
        operation_version: 1,
        outcome_identity: identity,
        idempotency_key: &[identity as u8; 32],
        branch: "main",
    })
    .expect("test correlation derives");
    WorthQueryDispatchOutboxRecord::from_installed_contract_for_test(
        correlation,
        &InstalledExternalEffectContract::Declared {
            correlation_family:
                worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                    "receipt-resolution-test",
                )
                .unwrap(),
            effect: "ReceiptResolutionEffect".to_owned(),
            rust_payload_type: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "worth.query.test.receipt-resolution-payload.v1",
            ),
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::new("test.receipt-resolution.payload"),
                BoundaryProtocolVersion::new(1),
            ),
            maximum_payload_bytes: 24,
            inbound: None,
        },
        vec![identity as u8; 8],
        identity,
    )
    .expect("declared test contract produces an outbox record")
}
