//! Persisted-row hostile evidence for the exact committed-outbox owner read.

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectValue, BoundaryProtocolIdentity, BoundaryProtocolVersion, InternedString,
};
use worth_query_declaration::facade::application_schema::ApplicationExternalEffectProtocol;
use worth_query_installation::facade::InstalledExternalEffectContract;
use worth_relational::facade::mvcc::BranchBoundRelationalTransaction;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, DeleteEntityIntent, EntityMutationIntent,
    EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch,
};

use super::owner_test_support::{release_commit_snapshot, retain_commit_basis};
use super::*;
use crate::domain_computation::application_aftermath::{
    bind_dispatch_outbox_create_intent, derive_external_effect_correlation_identity,
    ExternalEffectCorrelationBasis, WorthQueryDispatchOutboxRecord,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationCommitOutcomeIdentity, WorthQueryApplicationIdempotencyBinding,
    },
    primary_relational_branch_id,
    tests::fixture::installed_authorization_world,
    WorthQueryCommittedDispatchOutboxBinding,
};

#[test]
fn every_valid_persisted_field_substitution_breaks_whole_record_affinity() {
    for (field, replacement) in valid_field_substitutions() {
        let (provider, binding, commit, runtime_id) = committed_substituted_row(field, replacement);
        assert_eq!(
            provider.observe_expected(&binding, &commit, runtime_id),
            Err(Denial::RecordMismatch),
            "field {field} must participate in whole-record affinity"
        );
    }
}

#[test]
fn malformed_values_in_an_actual_committed_row_deny_before_record_copy_fallback() {
    for (field, replacement) in [
        (0, text("abc")),
        (3, text("UPPERCASE.IS.NOT.CANONICAL")),
        (4, AspectValue::UInt64(0)),
        (6, text("abc")),
    ] {
        let (provider, binding, commit, runtime_id) = committed_substituted_row(field, replacement);
        assert_eq!(
            provider.observe_expected(&binding, &commit, runtime_id),
            Err(Denial::Malformed),
            "malformed persisted field {field} must be decoded, not replaced from the binding"
        );
    }
}

#[test]
fn a_committed_record_of_another_kind_denies_before_projection() {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let record = record();
    let (binding, commit, runtime_id) = provider.graph.with_runtime_mut(|runtime| {
        let (outbox_intent, pending) = bind_dispatch_outbox_create_intent(
            Some(provider.graph.layout.provider_dispatch_outbox()),
            Some(&record),
            worth_relational::facade::identity::PartitionId::main(),
        )
        .unwrap();
        let idempotency = WorthQueryApplicationIdempotencyBinding::new([91; 32], [92; 32]);
        let idempotency_intent = super::super::idempotency::idempotency_create_intent(
            provider.graph.layout.provider_idempotency(),
            idempotency,
            WorthQueryApplicationCommitOutcomeIdentity::mint().unwrap(),
            0,
            worth_relational::facade::identity::PartitionId::main(),
        );
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
            WorkerIntentBatch::new("wrong-kind-outbox-owner-test")
                .push(outbox_intent)
                .push(idempotency_intent),
        ).expect("test staging stays within configured resource budgets");
        let committed = transaction.commit(runtime).unwrap();
        let correct = WorthQueryCommittedDispatchOutboxBinding::fixture_from_commit(
            provider.graph.layout.provider_dispatch_outbox(),
            Some(pending.record()),
            &committed,
        )
        .unwrap()
        .unwrap();
        let wrong_ref = committed
            .changed_records
            .iter()
            .find(|record_ref| *record_ref != correct.record_ref())
            .unwrap()
            .clone();
        let binding = WorthQueryCommittedDispatchOutboxBinding::fixture(record.clone(), wrong_ref);
        let commit = committed.outcome().commit.clone();
        retain_commit_basis(provider, runtime, &committed);
        release_commit_snapshot(runtime, &committed);
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_current_branch_snapshot(runtime, &primary_relational_branch_id())
            .unwrap();
        let runtime_id = snapshot.runtime_instance_id();
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        (binding, commit, runtime_id)
    });

    assert_eq!(
        provider.observe_expected(&binding, &commit, runtime_id),
        Err(Denial::WrongRecordKind)
    );
}

#[test]
fn a_deleted_record_is_non_visible_at_the_requested_commit_without_binding_fallback() {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let record = record();
    let (binding, deletion_commit, runtime_id) = provider.graph.with_runtime_mut(|runtime| {
        let (intent, pending) = bind_dispatch_outbox_create_intent(
            Some(provider.graph.layout.provider_dispatch_outbox()),
            Some(&record),
            worth_relational::facade::identity::PartitionId::main(),
        )
        .unwrap();
        let mut create: BranchBoundRelationalTransaction = {
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
        create.push_batch(WorkerIntentBatch::new("live-outbox-before-delete").push(intent)).expect("test staging stays within configured resource budgets");
        let created = create.commit(runtime).unwrap();
        let binding = WorthQueryCommittedDispatchOutboxBinding::fixture_from_commit(
            provider.graph.layout.provider_dispatch_outbox(),
            Some(pending.record()),
            &created,
        )
        .unwrap()
        .unwrap();
        let RecordRef::Entity(entity_id) = binding.record_ref().clone() else {
            panic!("outbox binding is an entity")
        };
        retain_commit_basis(provider, runtime, &created);
        release_commit_snapshot(runtime, &created);
        let mut delete: BranchBoundRelationalTransaction = {
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
        delete.push_batch(
            WorkerIntentBatch::new("delete-outbox-before-owner-read").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
            )),
        ).expect("test staging stays within configured resource budgets");
        let deleted = delete.commit(runtime).unwrap();
        let commit = deleted.outcome().commit.clone();
        retain_commit_basis(provider, runtime, &deleted);
        release_commit_snapshot(runtime, &deleted);
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_current_branch_snapshot(runtime, &primary_relational_branch_id())
            .unwrap();
        let runtime_id = snapshot.runtime_instance_id();
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        (binding, commit, runtime_id)
    });

    assert_eq!(
        provider.observe_expected(&binding, &deletion_commit, runtime_id),
        Err(Denial::Missing)
    );
}

fn committed_substituted_row(
    field: usize,
    replacement: AspectValue,
) -> (
    std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    WorthQueryCommittedDispatchOutboxBinding,
    worth_relational::facade::history::RelationalCommitReceipt,
    u64,
) {
    let world = installed_authorization_world(true);
    let provider = world.application.primary_provider.clone();
    let record = record();
    let layout = provider.graph.layout.provider_dispatch_outbox().clone();
    let branch = primary_relational_branch_id();
    let (binding, commit, runtime_id) = provider.graph.with_runtime_mut(|runtime| {
        let (intent, pending) =
            bind_dispatch_outbox_create_intent(
                Some(&layout),
                Some(&record),
                worth_relational::facade::identity::PartitionId::main(),
            )
            .unwrap();
        let MutationIntent::Create(CreateIntent::Entity(expected)) = intent.clone() else {
            panic!("outbox intent creates an entity")
        };
        let alternate_key = format!("corrupt-outbox-field-{field}");
        let corrupted = corrupted_spec(&layout, expected, field, replacement, &alternate_key);
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
            WorkerIntentBatch::new("persisted-outbox-corruption-matrix")
                .push(intent)
                .push(MutationIntent::Create(CreateIntent::Entity(corrupted))),
        ).expect("test staging stays within configured resource budgets");
        let committed = transaction.commit(runtime).unwrap();
        let corrupted_id = committed
            .created_entity(&CreatedEntityRef {
                partition_id: layout_partition(),
                kind_id: layout.entity_kind,
                client_key: worth_relational::facade::symbols::ClientKey::raw(
                    alternate_key.clone(),
                ),
            })
            .unwrap();
        let binding = WorthQueryCommittedDispatchOutboxBinding::fixture(
            pending.record().clone(),
            RecordRef::Entity(corrupted_id),
        );
        let commit = committed.outcome().commit.clone();
        retain_commit_basis(&provider, runtime, &committed);
        release_commit_snapshot(runtime, &committed);
        let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_current_branch_snapshot(runtime, &branch)
            .unwrap();
        let runtime_id = snapshot.runtime_instance_id();
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        (binding, commit, runtime_id)
    });
    (provider, binding, commit, runtime_id)
}

fn corrupted_spec(
    layout: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout,
    expected: EntitySpec,
    field: usize,
    replacement: AspectValue,
    client_key: &str,
) -> EntitySpec {
    let mut fields = expected
        .fields
        .iter()
        .map(|(locator, value)| (locator.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    fields.insert(field_locator(layout, field), replacement);
    EntitySpec {
        client_key: worth_relational::facade::symbols::ClientKey::raw(client_key),
        fields: AspectFieldPatch::from(fields),
        ..expected
    }
}

fn valid_field_substitutions() -> Vec<(usize, AspectValue)> {
    vec![
        (0, text("ab".repeat(32))),
        (1, text("other-family")),
        (2, text("OtherEffect")),
        (3, text("test.other.payload")),
        (4, AspectValue::UInt64(2)),
        (5, AspectValue::UInt64(999)),
        (6, text("ffee")),
        (7, AspectValue::UInt64(999)),
    ]
}

fn field_locator(
    layout: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout,
    field: usize,
) -> worth_foundational::facade::AspectFieldLocator {
    [
        &layout.correlation_locator,
        &layout.family_locator,
        &layout.effect_locator,
        &layout.protocol_identity_locator,
        &layout.protocol_version_locator,
        &layout.maximum_payload_bytes_locator,
        &layout.payload_locator,
        &layout.outcome_identity_locator,
    ][field]
        .clone()
}

fn record() -> WorthQueryDispatchOutboxRecord {
    let correlation = derive_external_effect_correlation_identity(ExternalEffectCorrelationBasis {
        correlation_family:
            worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                "corruption-owner-test",
            )
            .unwrap(),
        operation_slot: "notify",
        operation_version: 1,
        outcome_identity: 41,
        idempotency_key: &[41; 32],
        branch: "main",
    })
    .unwrap();
    WorthQueryDispatchOutboxRecord::from_installed_contract(
        correlation,
        &InstalledExternalEffectContract::Declared {
            correlation_family:
                worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(
                    "corruption-owner-test",
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
        },
        vec![41; 8],
        41,
    )
    .unwrap()
}

fn layout_partition() -> worth_relational::facade::identity::PartitionId {
    worth_relational::facade::identity::PartitionId::main()
}

fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::from(value.into()))
}
