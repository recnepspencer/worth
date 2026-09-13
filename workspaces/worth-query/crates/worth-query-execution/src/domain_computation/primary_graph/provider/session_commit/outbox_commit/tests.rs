//! Owner-local proofs for commit-to-receipt outbox resolution.

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::ApplicationExternalEffectProtocol;
use worth_query_installation::facade::InstalledExternalEffectContract;
use worth_relational::facade::transactions::{RecordRef, WorkerIntentBatch};

use super::{
    WorthQueryCommittedDispatchOutboxBindingDenial, WorthQueryCommittedDispatchOutboxResolution,
};
use crate::domain_computation::application_aftermath::{
    bind_dispatch_outbox_create_intent, derive_external_effect_correlation_identity,
    ExternalEffectCorrelationBasis, WorthQueryDispatchOutboxRecord,
    WorthQueryPendingDispatchOutbox,
};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;

#[test]
fn real_commit_mapping_denial_reaches_receipt_projection() {
    let evidence = commit_only_other_outbox();
    let seal = evidence
        .committed_resolution
        .seal_for_receipt()
        .expect("the committed create resolves");
    let binding = seal
        .binding()
        .expect("the committed create has one binding");
    assert_eq!(binding.record(), evidence.committed_pending.record());
    assert_eq!(binding.record_ref(), &evidence.expected_record_ref);

    assert_eq!(
        evidence
            .requested_resolution
            .seal_for_receipt()
            .map(|seal| seal.into_binding()),
        Err(WorthQueryCommittedDispatchOutboxBindingDenial::CreatedEntityMissing)
    );
}

#[test]
fn real_commit_with_no_requested_outbox_projects_honest_absence() {
    let evidence = commit_only_other_outbox();
    assert_eq!(
        evidence
            .absent_resolution
            .seal_for_receipt()
            .expect("honest absence is a successful receipt projection")
            .into_binding(),
        None
    );
}

struct CommittedOutboxResolutionEvidence {
    committed_pending: WorthQueryPendingDispatchOutbox,
    committed_resolution: WorthQueryCommittedDispatchOutboxResolution,
    requested_resolution: WorthQueryCommittedDispatchOutboxResolution,
    absent_resolution: WorthQueryCommittedDispatchOutboxResolution,
    expected_record_ref: RecordRef,
}

fn commit_only_other_outbox() -> CommittedOutboxResolutionEvidence {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let (_, requested) = bind_dispatch_outbox_create_intent(
        Some(provider.graph.layout.provider_dispatch_outbox()),
        Some(&record(1)),
        worth_relational::facade::identity::PartitionId::main(),
    )
    .expect("requested outbox create binds");
    let (committed_intent, committed_pending) = bind_dispatch_outbox_create_intent(
        Some(provider.graph.layout.provider_dispatch_outbox()),
        Some(&record(2)),
        worth_relational::facade::identity::PartitionId::main(),
    )
    .expect("committed outbox create binds");
    provider.graph.with_runtime_mut(|runtime| {
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
            )
            .expect("test staging stays within configured resource budgets");
        let committed = transaction
            .commit(runtime)
            .expect("the other outbox commits");
        let expected_record_ref = RecordRef::Entity(
            committed
                .created_entity(committed_pending.created_entity())
                .expect("the exact committed create has an owner-minted mapping"),
        );
        let evidence = CommittedOutboxResolutionEvidence {
            committed_resolution: WorthQueryCommittedDispatchOutboxResolution::from_commit(
                Some(&committed_pending),
                &committed,
            ),
            requested_resolution: WorthQueryCommittedDispatchOutboxResolution::from_commit(
                Some(&requested),
                &committed,
            ),
            absent_resolution: WorthQueryCommittedDispatchOutboxResolution::from_commit(
                None, &committed,
            ),
            committed_pending,
            expected_record_ref,
        };
        crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
        evidence
    })
}

fn record(identity: u64) -> WorthQueryDispatchOutboxRecord {
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
    WorthQueryDispatchOutboxRecord::from_installed_contract(
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
        },
        vec![identity as u8; 8],
        identity,
    )
    .expect("declared test contract produces an outbox record")
}
