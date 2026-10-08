use crate::tests::support::*;

#[test]
fn branch_head_index_moves_at_cutover_before_reverse_settlement() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "head-index-anchor");
    fork_publication_branch(&mut runtime, "head-index-a");
    fork_publication_branch(&mut runtime, "head-index-b");

    let mut first_transaction = test_owner_begin_transaction_for_main(&runtime);
    first_transaction
        .push_batch(
            batch_create("head-index-first"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("first publication stages");
    let first_candidate = runtime
        .prepare_branch_transaction(
            first_transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("first publication prepares");
    let crate::mvcc::RelationalPublicationOutcome::Performed(first) = runtime
        .publication_port()
        .compare_and_publish(first_candidate)
    else {
        panic!("first publication performs");
    };
    let first_version = first.canonical_commit().commit.version_id;

    let branch_a = prepare_publication(&mut runtime, "head-index-a", "head-index-a-write");
    let crate::mvcc::RelationalPublicationOutcome::Performed(branch_a) =
        runtime.publication_port().compare_and_publish(branch_a)
    else {
        panic!("branch A publication performs");
    };
    let branch_b = prepare_publication(&mut runtime, "head-index-b", "head-index-b-write");
    let crate::mvcc::RelationalPublicationOutcome::Performed(branch_b) =
        runtime.publication_port().compare_and_publish(branch_b)
    else {
        panic!("branch B publication performs");
    };
    assert_eq!(
        runtime.history.oldest_branch_head_version(),
        Some(first_version),
        "all performed cutovers reach the head index before any settlement"
    );

    let committed_b = runtime
        .settle_performed_publication(branch_b)
        .expect("branch B settles first");
    release_test_commit_snapshot(&runtime, &committed_b);
    let committed_a = runtime
        .settle_performed_publication(branch_a)
        .expect("branch A settles second");
    release_test_commit_snapshot(&runtime, &committed_a);
    let committed_main = runtime
        .settle_performed_publication(first)
        .expect("main settles last");
    release_test_commit_snapshot(&runtime, &committed_main);
    assert_eq!(
        runtime.history.oldest_branch_head_version(),
        Some(first_version),
        "reverse settlement cannot move branch-head truth backward"
    );
}

#[test]
fn preparation_order_does_not_reserve_a_subscriber_stream_gap() {
    let mut runtime = runtime_with_test_schema();
    create_entity(&runtime, "stream-order-anchor");
    fork_publication_branch(&mut runtime, "prepared-first");
    fork_publication_branch(&mut runtime, "published-first");

    let prepared_first = prepare_publication(&mut runtime, "prepared-first", "write-a");
    let published_first = prepare_publication(&mut runtime, "published-first", "write-b");
    let published_first = match runtime
        .publication_port()
        .compare_and_publish(published_first)
    {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("candidate B publishes first: {outcome:?}"),
    };
    let position_b = published_first.patch_position();
    let before_a = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: Some(position_b),
            max_commits: usize::MAX,
        })
        .expect("B is immediately a valid resume position");
    assert!(before_a.patches.is_empty());

    let prepared_first = match runtime
        .publication_port()
        .compare_and_publish(prepared_first)
    {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("candidate A publishes after B: {outcome:?}"),
    };
    assert!(
        prepared_first.canonical_commit().commit.commit_id
            < published_first.canonical_commit().commit.commit_id
    );
    assert!(position_b < prepared_first.patch_position());

    let after_a = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: Some(position_b),
            max_commits: usize::MAX,
        })
        .expect("subscriber resumes through the later A publication");
    assert_eq!(after_a.patches.len(), 1);
    assert_eq!(after_a.patches[0].position, prepared_first.patch_position());
    assert_eq!(
        after_a.latest_commit_id,
        Some(prepared_first.canonical_commit().commit.commit_id)
    );
    let committed_b = runtime
        .settle_performed_publication(published_first)
        .expect("published-first branch settles explicitly");
    release_test_commit_snapshot(&runtime, &committed_b);
    let committed_a = runtime
        .settle_performed_publication(prepared_first)
        .expect("prepared-first branch settles explicitly");
    release_test_commit_snapshot(&runtime, &committed_a);
}

#[test]
fn checkpoint_tail_recovery_follows_publication_order_with_exact_reserved_identities() {
    let mut runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "recovery-order-anchor");
    fork_publication_branch(&mut runtime, "prepared-first");
    fork_publication_branch(&mut runtime, "published-first");

    let prepared_first = prepare_publication(&mut runtime, "prepared-first", "write-a");
    let published_first = prepare_publication(&mut runtime, "published-first", "write-b");
    let published_first = match runtime
        .publication_port()
        .compare_and_publish(published_first)
    {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("candidate B publishes first: {outcome:?}"),
    };
    let published_first_id = published_first.canonical_commit().commit.commit_id;
    let published_first_position = published_first.patch_position();
    let published_first_entity = created_entity_from_performed(&published_first);
    let committed_b = runtime
        .settle_performed_publication(published_first)
        .expect("B settles before the checkpoint");
    release_test_commit_snapshot(&runtime, &committed_b);
    runtime
        .durability_authority()
        .checkpoint()
        .expect("checkpoint covers B by performed stream position");

    let prepared_first = match runtime
        .publication_port()
        .compare_and_publish(prepared_first)
    {
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("candidate A publishes after the checkpoint: {outcome:?}"),
    };
    let prepared_first_id = prepared_first.canonical_commit().commit.commit_id;
    let prepared_first_position = prepared_first.patch_position();
    let prepared_first_entity = created_entity_from_performed(&prepared_first);
    assert!(prepared_first_id < published_first_id);
    assert!(prepared_first_position > published_first_position);
    let committed_a = runtime
        .settle_performed_publication(prepared_first)
        .expect("A settles after the checkpoint");
    release_test_commit_snapshot(&runtime, &committed_a);

    let plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    assert_eq!(plan.tail_log.len(), 1);
    assert_eq!(plan.tail_log[0].position(), prepared_first_position);
    assert_eq!(
        plan.tail_log[0].envelope().commit.commit_id,
        prepared_first_id
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(plan)
        .expect("fresh recovery replays A after checkpointed B");
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("published-first".to_owned()))
            .expect("B branch head recovers from checkpoint")
            .commit_id,
        published_first_id
    );
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("prepared-first".to_owned()))
            .expect("A branch head recovers from the position-selected tail")
            .commit_id,
        prepared_first_id
    );
    assert_eq!(
        observed_entity_name(&recovered, "prepared-first", prepared_first_entity),
        Some("write-a".to_owned())
    );
    assert_eq!(
        observed_entity_name(&recovered, "prepared-first", published_first_entity),
        None
    );
    assert_eq!(
        observed_entity_name(&recovered, "published-first", published_first_entity),
        Some("write-b".to_owned())
    );
    assert_eq!(
        observed_entity_name(&recovered, "published-first", prepared_first_entity),
        None
    );
    assert_eq!(
        observed_entity_name(&recovered, "main", prepared_first_entity),
        None
    );
    assert_eq!(
        observed_entity_name(&recovered, "main", published_first_entity),
        None
    );
    let continued = create_entity_outcome(&recovered, "recovery-order-continued");
    assert!(continued.commit.commit_id > published_first_id);
    assert!(continued.patch_position() > prepared_first_position);
    release_test_commit_snapshot(&recovered, &continued);
}

fn created_entity_from_performed(
    performed: &crate::facade::mvcc::PerformedRelationalCommit,
) -> crate::facade::identity::EntityId {
    let patch = performed
        .canonical_commit()
        .patch
        .authoritative_record_patches
        .first()
        .expect("one created entity patch");
    match &patch.target {
        crate::transactions::data::RecordRef::Entity(entity) => *entity,
        crate::transactions::data::RecordRef::Relation(_) => panic!("expected entity patch"),
    }
}

fn observed_entity_name(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
    entity: crate::facade::identity::EntityId,
) -> Option<String> {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("observed branch identity exists");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("branch observation is owner-admitted");
    let read = runtime
        .begin_branch_transaction(
            &basis,
            crate::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("observation transaction binds to the admitted root")
        .read_entity(entity)
        .expect("entity read is admitted on the observed root");
    read.base().and_then(read_entity_name)
}

fn prepare_publication(
    runtime: &mut crate::facade::runtime::RelationalRuntime,
    branch: &str,
    entity: &str,
) -> crate::facade::mvcc::PreparedRelationalCommitCandidate {
    let mut transaction = begin_publication_transaction(runtime, branch);
    transaction
        .push_batch(
            batch_create(entity),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    runtime
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("publication candidate prepares")
}

fn fork_publication_branch(runtime: &mut crate::facade::runtime::RelationalRuntime, branch: &str) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main has an exact fork source");
    runtime
        .fork_branch(BranchId(branch.to_owned()), source)
        .expect("publication branch fork succeeds");
}

fn begin_publication_transaction(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: &str,
) -> crate::facade::mvcc::BranchBoundRelationalTransaction {
    let identity = runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("publication branch identity exists");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("publication branch basis is admitted");
    runtime
        .begin_branch_transaction(
            &basis,
            crate::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("publication transaction binds")
}
