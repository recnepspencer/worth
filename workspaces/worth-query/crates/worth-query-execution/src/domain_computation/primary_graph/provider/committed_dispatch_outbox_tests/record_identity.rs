use super::*;

#[test]
fn unrelated_identical_row_cannot_make_owner_mapping_ambiguous() {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let record = record_for(5);
    provider.graph.with_runtime_mut(|runtime| {
        let (intent, pending) =
            crate::domain_computation::application_aftermath::bind_dispatch_outbox_create_intent(
                Some(provider.graph.layout.provider_dispatch_outbox()),
                Some(&record),
                worth_relational::facade::identity::PartitionId::main(),
            )
            .unwrap();
        let MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Entity(
            mut unrelated,
        )) = intent.clone()
        else {
            panic!("outbox fixture creates an entity")
        };
        unrelated.client_key = worth_relational::facade::symbols::ClientKey::raw(
            "unrelated-row-with-identical-fields",
        );
        let unrelated_created = worth_relational::facade::transactions::CreatedEntityRef {
            partition_id: unrelated.partition_id,
            kind_id: unrelated.kind_id,
            client_key: unrelated.client_key.clone(),
        };
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
        transaction
            .push_batch(
                WorkerIntentBatch::new("owner-mapped-committed-outbox-test")
                    .push(intent)
                    .push(MutationIntent::Create(
                        worth_relational::facade::transactions::CreateIntent::Entity(unrelated),
                    )),
            )
            .expect("test staging stays within configured resource budgets");
        let committed = transaction.commit(runtime).unwrap();
        assert_ne!(
            committed.created_entity(pending.created_entity()),
            committed.created_entity(&unrelated_created),
            "each exact create reference retains its own owner-minted identity"
        );
        let binding = WorthQueryCommittedDispatchOutboxBinding::fixture_from_commit(
            provider.graph.layout.provider_dispatch_outbox(),
            Some(pending.record()),
            &committed,
        )
        .unwrap()
        .unwrap();
        assert_eq!(binding.record(), &record);
        assert_eq!(
            binding.record_ref(),
            &RecordRef::Entity(committed.created_entity(pending.created_entity()).unwrap(),)
        );
        assert_eq!(
            WorthQueryCommittedDispatchOutboxBinding::fixture_from_commit(
                provider.graph.layout.provider_dispatch_outbox(),
                Some(&record_for(6)),
                &committed,
            ),
            Err(WorthQueryCommittedDispatchOutboxBindingDenial::CreatedEntityMissing)
        );
        retain_commit_basis(provider, runtime, &committed);
        release_commit_snapshot(runtime, &committed);
    });
}

#[test]
fn another_committed_record_ref_cannot_substitute_for_the_bound_outbox() {
    let world = installed_authorization_world(true);
    let provider = &world.application.primary_provider;
    let first = record_for(61);
    let second = record_for(62);
    let (first_binding, second_binding, commit, runtime_id) =
        provider.graph.with_runtime_mut(|runtime| {
            let intent = |record: &WorthQueryDispatchOutboxRecord| {
                dispatch_outbox_create_intent(
                    Some(provider.graph.layout.provider_dispatch_outbox()),
                    Some(record),
                )
                .unwrap()
            };
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
                    WorkerIntentBatch::new("record-ref-substitution-owner-test")
                        .push(intent(&first))
                        .push(intent(&second)),
                )
                .expect("test staging stays within configured resource budgets");
            let committed = transaction.commit(runtime).unwrap();
            let bind = |record| {
                WorthQueryCommittedDispatchOutboxBinding::fixture_from_commit(
                    provider.graph.layout.provider_dispatch_outbox(),
                    Some(record),
                    &committed,
                )
                .unwrap()
                .unwrap()
            };
            let commit = committed.outcome().commit.clone();
            retain_commit_basis(provider, runtime, &committed);
            release_commit_snapshot(runtime, &committed);
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_current_branch_snapshot(runtime, &primary_relational_branch_id())
                .unwrap();
            let runtime_id = snapshot.runtime_instance_id();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            (bind(&first), bind(&second), commit, runtime_id)
        });
    let substituted = WorthQueryCommittedDispatchOutboxBinding::fixture(
        first_binding.record().clone(),
        second_binding.record_ref().clone(),
    );
    assert_eq!(
        provider.observe_expected(&substituted, &commit, runtime_id),
        Err(Denial::RecordMismatch)
    );
}
