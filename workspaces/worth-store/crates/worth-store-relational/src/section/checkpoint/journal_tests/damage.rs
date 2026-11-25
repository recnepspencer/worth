//! End-to-end damaged journal record through protected envelope demand proves
//! sticky `Cause::Damaged` with zero second-read Store contacts.
use super::*;
use worth_relational::facade::{
    publication::PatchStreamPosition, storage::RelationalStorageUnavailableCause as Cause,
    transactions::RelationalExecutionDenialCause,
};

#[test]
fn damaged_protected_journal_envelope_records_sticky_damage_without_recontacting_store() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let source = StoreColdSource::new(serving.clone());
    let selected = Arc::new(Selected::default());
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = &ExecutionRequest::serial(&serial);
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "damaged.journal".into(),
            schema_id: SchemaId("damaged.journal".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let producer = runtime(&serving, &source, &selected, &schema);
    let create = |owner: &RelationalRuntime, key: &str| {
        let (_, basis) = owner.observe_branch(&owner.main_branch_identity()).unwrap();
        let mut transaction = owner
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                    EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: key.into(),
                        fields: Default::default(),
                    },
                ))),
                execution.allocation_policy(),
            )
            .unwrap();
        transaction.commit(execution, owner).unwrap()
    };

    // Commit 1 uses a large extent-backed payload; commit 2 moves head past it
    // so commit 1 is a cold non-head `EnvelopeBacking::Journal` entry on reopen.
    let first = create(&producer, &"damaged-journal-extent-".repeat(8192));
    producer
        .snapshots()
        .release_snapshot(&first.snapshot)
        .unwrap();
    drop(first);
    let second = create(&producer, "second-commit");
    producer
        .snapshots()
        .release_snapshot(&second.snapshot)
        .unwrap();
    drop(second);

    // Capture the healthy journal tail and commit 1's physical chunk before closing Store.
    let original = journal_frame(&serving, 1);
    let tail = crate::recovered_history::journal_tail(&serving, 0).unwrap();
    let transitions = crate::recovered_transitions::transition_tail(&serving, 0).unwrap();
    drop(producer);
    drop(source);
    let closed = Arc::try_unwrap(serving).ok().unwrap().close();
    drop(closed);

    // Corrupt commit 1's on-disk extent payload while Store is closed so the
    // reopened physical runtime has no cached clean copy of commit 1.
    let damaged = corrupt_original(directory.path(), &original.first_chunk);
    assert!(
        damaged > 0,
        "commit 1 physical payload chunk must be corrupted on disk"
    );

    // Reopen Store cleanly (empty cache) and recover the cold journal-backed catalog.
    let (format, _, access) = physical::configuration();
    let media = physical::media(directory.path());
    let registry = OrderedJournalRegistry::declare(
        3,
        &[RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS],
        &["home.checkpoint"],
    )
    .unwrap();
    let admission = PhysicalRecordOpen::new(format, access, physical::durability(&media))
        .with_journal_registry(registry);
    let serving = Arc::new(physical::success(media.open_record_store(admission)));
    let source = StoreColdSource::new(serving.clone());
    let mut recovered = runtime(&serving, &source, &selected, &schema);
    let recovery = RelationalDurableRecoverySource::new(None, tail, transitions, source.clone());
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    drop(authority);
    recovered.run_branch_root_reclamation_pass();

    let receipt = recovered
        .history()
        .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(1))
        .unwrap();

    // 1st demand hits protected journal read -> Store checksum failure -> Cause::Damaged.
    let reads_before = source.counters().commit_reads;
    let first_err = recovered
        .history()
        .historical_committed_version(execution, receipt.version_id)
        .unwrap_err();
    let reads_after_first = source.counters().commit_reads;
    assert_eq!(
        reads_after_first,
        reads_before + 1,
        "first protected envelope demand must contact Store once"
    );
    let RelationalExecutionDenialCause::StorageUnavailable(first_failure) = first_err else {
        panic!("expected StorageUnavailable(Damaged) on first read, got {first_err:?}");
    };
    assert_eq!(first_failure.cause, Cause::Damaged);

    // 2nd demand returns the exact same sticky Cause::Damaged with ZERO Store contacts.
    let second_err = recovered
        .history()
        .historical_committed_version(execution, receipt.version_id)
        .unwrap_err();
    assert_eq!(
        source.counters().commit_reads,
        reads_after_first,
        "second envelope demand must return sticky damage without contacting Store again"
    );
    let RelationalExecutionDenialCause::StorageUnavailable(second_failure) = second_err else {
        panic!("expected sticky StorageUnavailable(Damaged) on second read, got {second_err:?}");
    };
    assert_eq!(second_failure, first_failure);

    drop(recovered);
    drop(source);
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
}

#[test]
fn transient_busy_on_protected_journal_read_retries_outside_envelope_lock_and_respects_cancel() {
    use std::any::Any;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use worth_relational::facade::durability::{
        RelationalDurableColdSource, RelationalDurableMemberCustody,
        RelationalDurableMemberLocator, RelationalDurableOwnedReadRefusal as Refusal,
        RelationalDurableReadBounds as Bounds, RelationalDurableReadPayload,
    };

    #[derive(Debug)]
    struct BusyWrappingSource {
        inner: Arc<StoreColdSource>,
        remaining_busy: AtomicUsize,
        attempts: AtomicUsize,
        cancel_on_busy: Mutex<Option<CancellationSource>>,
    }

    impl RelationalDurableColdSource for BusyWrappingSource {
        fn retain_members(
            &self,
            members: &[RelationalDurableMemberLocator],
        ) -> Result<RelationalDurableMemberCustody, Cause> {
            self.inner.retain_members(members)
        }
        fn read_member(
            &self,
            locator: RelationalDurableMemberLocator,
            limit: u64,
        ) -> Result<Vec<u8>, Cause> {
            self.inner.read_member(locator, limit)
        }
        fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
            self.inner.read_commit(position, limit)
        }
        fn journal_read_protection(
            &self,
            request: &ExecutionRequest<'_, '_>,
        ) -> Result<(Bounds, RelationalDurableReadPayload), Refusal> {
            self.attempts.fetch_add(1, Ordering::SeqCst);
            let prev = self
                .remaining_busy
                .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1));
            if prev.is_ok() {
                if let Some(cancel) = self.cancel_on_busy.lock().unwrap().take() {
                    cancel.cancel();
                }
                return Err(Refusal::Busy);
            }
            self.inner.journal_read_protection(request)
        }
        fn read_commit_owned(
            &self,
            request: &ExecutionRequest<'_, '_>,
            protection: &(dyn Any + Send + Sync),
            position: PatchStreamPosition,
            limit: u64,
        ) -> Result<worth_execution::ExecutionImmutableBytes, Refusal> {
            self.inner
                .read_commit_owned(request, protection, position, limit)
        }
    }

    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(test_store::initialize(directory.path()));
    let inner = StoreColdSource::new(serving.clone());
    let selected = Arc::new(Selected::default());
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = &ExecutionRequest::serial(&serial);
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "busy.journal".into(),
            schema_id: SchemaId("busy.journal".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let producer = runtime(&serving, &inner, &selected, &schema);
    let create = |owner: &RelationalRuntime, key: &str| {
        let (_, basis) = owner.observe_branch(&owner.main_branch_identity()).unwrap();
        let mut transaction = owner
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                    EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: key.into(),
                        fields: Default::default(),
                    },
                ))),
                execution.allocation_policy(),
            )
            .unwrap();
        let outcome = transaction.commit(execution, owner).unwrap();
        owner
            .snapshots()
            .release_snapshot(&outcome.snapshot)
            .unwrap();
    };
    create(&producer, "busy-first");
    create(&producer, "busy-second");
    let tail = crate::recovered_history::journal_tail(&serving, 0).unwrap();
    let transitions = crate::recovered_transitions::transition_tail(&serving, 0).unwrap();
    drop(producer);

    let busy_source = Arc::new(BusyWrappingSource {
        inner: inner.clone(),
        remaining_busy: AtomicUsize::new(2),
        attempts: AtomicUsize::new(0),
        cancel_on_busy: Mutex::new(None),
    });
    let mut recovered = runtime(&serving, &inner, &selected, &schema);
    let recovery =
        RelationalDurableRecoverySource::new(None, tail, transitions, busy_source.clone());
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    drop(authority);
    recovered.run_branch_root_reclamation_pass();

    let receipt = recovered
        .history()
        .immutable_commit_receipt_at_patch_stream_position(PatchStreamPosition(1))
        .unwrap();
    let commit = recovered
        .history()
        .historical_committed_version(execution, receipt.version_id)
        .unwrap()
        .expect("committed version summary");
    assert_eq!(commit.commit().version_id, receipt.version_id);
    assert_eq!(busy_source.attempts.load(Ordering::SeqCst), 3);

    // Cancelled request stops cleanly as Cancelled instead of spinning or returning SourceUnavailable.
    let cancel = CancellationSource::new();
    let cancel_serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancel.token(), None);
    let cancel_exec = ExecutionRequest::serial(&cancel_serial);
    busy_source.remaining_busy.store(100, Ordering::SeqCst);
    *busy_source.cancel_on_busy.lock().unwrap() = Some(cancel);
    recovered.run_branch_root_reclamation_pass();
    let err = recovered
        .history()
        .historical_committed_version(&cancel_exec, receipt.version_id)
        .unwrap_err();
    assert!(
        matches!(err, RelationalExecutionDenialCause::Cancelled),
        "cancelled read during transient Busy must stop as Cancelled, got {err:?}"
    );
    drop((commit, recovered, busy_source, inner));
    let _closed = Arc::try_unwrap(serving).ok().unwrap().close();
}
