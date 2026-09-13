use worth_foundational::facade::AspectValue;

#[path = "committed_dispatch_outbox_tests/record_identity.rs"]
mod record_identity;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::mvcc::BranchBoundRelationalTransaction;
use worth_relational::facade::transactions::{
    AspectFieldPatch, DeleteEntityIntent, EntityMutationIntent, MutationIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::owner_test_support::{
    commit_record, record_for, release_commit_snapshot, retain_commit_basis, string,
};
use super::restoration::hex_bytes;
use super::*;
use crate::domain_computation::application_aftermath::dispatch_outbox_create_intent;
use crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxBindingDenial;
use crate::domain_computation::primary_graph::{
    primary_relational_branch_id, tests::fixture::installed_authorization_world,
    WorthQueryCommittedDispatchOutboxBinding,
};

#[test]
fn owner_read_denies_missing_foreign_and_every_commit_affinity_substitution() {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let (binding, commit, runtime_id) = commit_record(provider, 1);
    let record = binding.record().clone();
    assert_eq!(
        provider
            .observe_expected(&binding, &commit, runtime_id)
            .unwrap()
            .record(),
        &record
    );

    let absent = WorthQueryCommittedDispatchOutboxBinding::fixture(
        record_for(2),
        RecordRef::Entity(worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            u64::MAX,
            1,
        )),
    );
    assert_eq!(
        provider.observe_expected(&absent, &commit, runtime_id),
        Err(Denial::Missing)
    );
    assert_eq!(
        provider.observe_expected(&binding, &commit, runtime_id + 1),
        Err(Denial::ForeignRuntime)
    );
    assert_commit_affinity_substitutions(provider, &binding, commit, runtime_id);
}

fn assert_commit_affinity_substitutions(
    provider: &WorthQueryPrimaryGraphProvider,
    binding: &WorthQueryCommittedDispatchOutboxBinding,
    commit: worth_relational::facade::history::RelationalCommitReceipt,
    runtime_id: u64,
) {
    let mut wrong_commit_id = commit.clone();
    wrong_commit_id.commit_id = CommitId(commit.commit_id.0.saturating_sub(1));
    assert_eq!(
        provider.observe_expected(binding, &wrong_commit_id, runtime_id),
        Err(Denial::ExactCommitUnavailable)
    );
    let mut wrong_version = commit.clone();
    wrong_version.version_id = VersionId(commit.version_id.0.saturating_sub(1));
    assert_eq!(
        provider.observe_expected(binding, &wrong_version, runtime_id),
        Err(Denial::CommitMismatch)
    );
    let feature = BranchId("committed-outbox-feature".to_owned());
    provider.graph.with_runtime_mut(|runtime| {
        let (_, basis) = runtime.observe_fork_source(&commit.branch_id).unwrap();
        runtime.fork_branch(feature.clone(), basis).unwrap();
        let feature_record = record_for(99);
        let mut transaction = {
            let identity = runtime
                .branch_identity(&feature)
                .expect("feature branch identity");
            let transaction_validation_input = runtime
                .admit_branch_basis(&identity)
                .expect("feature branch binding");
            runtime
                .begin_branch_transaction(
                    &transaction_validation_input,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("owner-admitted transaction context")
        };
        transaction
            .push_batch(
                WorkerIntentBatch::new("feature-outbox-head").push(
                    dispatch_outbox_create_intent(
                        Some(provider.graph.layout.provider_dispatch_outbox()),
                        Some(&feature_record),
                    )
                    .unwrap(),
                ),
            )
            .expect("test staging stays within configured resource budgets");
        let committed = transaction.commit(runtime).unwrap();
        retain_commit_basis(provider, runtime, &committed);
        release_commit_snapshot(runtime, &committed);
    });
    let mut wrong_branch = commit;
    wrong_branch.branch_id = feature;
    assert_eq!(
        provider.observe_expected(binding, &wrong_branch, runtime_id),
        Err(Denial::CommitMismatch)
    );
}

#[test]
fn fresh_later_head_still_reports_the_rows_exact_creation_commit() {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let (first_binding, first_commit, runtime_id) = commit_record(provider, 3);
    let first = first_binding.record().clone();
    let (_, later_commit, _) = commit_record(provider, 4);
    assert_ne!(first_commit, later_commit);

    let observed = provider
        .observe_expected(&first_binding, &first_commit, runtime_id)
        .expect("the current snapshot retains the older live outbox row");
    assert_eq!(observed.commit_reference(), &first_commit);
    assert_eq!(observed.record(), &first);
    assert_eq!(observed.work().exact_commit_snapshots(), 1);
    assert_eq!(observed.work().canonical_version_probes(), 1);
    assert_eq!(observed.work().projection_views(), 1);
    assert_eq!(observed.work().examined_index_entries(), 0);
    assert_eq!(observed.work().direct_record_probes(), 1);
    assert_eq!(observed.work().projected_records(), 1);
    assert_eq!(observed.work().projected_fields(), 8);
    assert_eq!(observed.work().reconstruction_requests(), 0);
}

#[test]
fn every_later_valid_field_substitution_leaves_exact_commit_truth_unchanged() {
    for (field, replacement) in valid_later_field_substitutions() {
        let world = installed_authorization_world(true);
        let provider = &world.application.primary_provider;
        let (binding, commit, runtime_id) = commit_record(provider, 31);
        let record = binding.record().clone();
        let observed = provider
            .observe_expected(&binding, &commit, runtime_id)
            .expect("original exact-commit row");
        let RecordRef::Entity(entity_id) = observed.record_ref() else {
            panic!("dispatch outbox is an entity record");
        };
        provider.graph.with_runtime_mut(|runtime| {
            let locator =
                outbox_field_locator(provider.graph.layout.provider_dispatch_outbox(), field);
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
                    WorkerIntentBatch::new("later-valid-outbox-substitution").push(
                        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                            UpdateEntityFieldsIntent {
                                entity_id: *entity_id,
                                fields: AspectFieldPatch::from_locator(locator, replacement),
                            },
                        )),
                    ),
                )
                .expect("test staging stays within configured resource budgets");
            let committed = transaction.commit(runtime).unwrap();
            retain_commit_basis(provider, runtime, &committed);
            release_commit_snapshot(runtime, &committed);
        });

        let exact = provider
            .observe_expected(&binding, &commit, runtime_id)
            .expect("later head cannot replace exact-commit truth");
        assert_eq!(exact.record(), &record);
        assert_eq!(exact.commit_reference(), &commit);
    }
}

#[test]
fn later_deletion_cannot_erase_exact_commit_truth() {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let (binding, commit, runtime_id) = commit_record(provider, 32);
    let record = binding.record().clone();
    let observed = provider
        .observe_expected(&binding, &commit, runtime_id)
        .expect("original exact-commit row");
    let RecordRef::Entity(entity_id) = observed.record_ref() else {
        panic!("dispatch outbox is an entity record");
    };
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
            .push_batch(WorkerIntentBatch::new("later-outbox-deletion").push(
                MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                    entity_id: *entity_id,
                })),
            ))
            .expect("test staging stays within configured resource budgets");
        let committed = transaction.commit(runtime).unwrap();
        retain_commit_basis(provider, runtime, &committed);
        release_commit_snapshot(runtime, &committed);
    });

    let exact = provider
        .observe_expected(&binding, &commit, runtime_id)
        .expect("later deletion cannot erase retained exact-commit truth");
    assert_eq!(exact.record(), &record);
}

fn valid_later_field_substitutions() -> Vec<(usize, AspectValue)> {
    vec![
        (0, string(hex_bytes(&[9; 32]))),
        (1, string("later-family".to_owned())),
        (2, string("later-effect".to_owned())),
        (3, string("test.owner.later".to_owned())),
        (4, AspectValue::UInt64(2)),
        (5, AspectValue::UInt64(25)),
        (6, string("ff".to_owned())),
        (7, AspectValue::UInt64(99)),
    ]
}

fn outbox_field_locator(
    layout: &WorthQueryDispatchOutboxLayout,
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
