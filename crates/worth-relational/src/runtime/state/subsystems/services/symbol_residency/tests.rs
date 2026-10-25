use super::*;
use crate::durability::data::{
    RelationalCheckpointDirectory, RelationalCheckpointManifest, RelationalCheckpointMember,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::tests::support::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Debug)]
struct PageSource {
    members: BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    reads: AtomicUsize,
    corrupt: AtomicBool,
}
impl RelationalDurableColdSource for PageSource {
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        if locator.length > limit {
            return Err(Cause::LimitExceeded {
                length: locator.length,
                limit,
            });
        }
        self.reads.fetch_add(1, Ordering::SeqCst);
        let mut bytes = self
            .members
            .get(&locator)
            .ok_or(Cause::SourceUnavailable)?
            .to_vec();
        if self.corrupt.load(Ordering::SeqCst) {
            bytes[0] ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}

fn dictionary() -> (
    RuntimeSymbolTable,
    RelationalCheckpointDirectory,
    Arc<PageSource>,
    crate::symbols::data::SymbolTableSnapshot,
) {
    use crate::facade::mvcc::WorkerIntentBatch;
    use crate::facade::transactions::{BulkEntityCreateIntent, CreateIntent, MutationIntent};
    use crate::identity::data::{KindId, PartitionId};
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("paged-symbol-source").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys: (0..600)
                        .map(|index| {
                            crate::symbols::data::ClientKey::raw(format!("paged-key-{index:04}"))
                        })
                        .collect(),
                    field_patches: (0..600)
                        .map(|index| name_field_patch(&format!("paged-value-{index:04}")))
                        .collect(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let outcome = transaction.commit(&execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &outcome);
    let native = runtime
        .durability_authority()
        .native_checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    #[derive(serde::Deserialize)]
    struct DirectoryInput {
        manifest: RelationalCheckpointManifest,
    }
    let input: DirectoryInput = rmp_serde::from_slice(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let source = Arc::new(PageSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("unpublished memory source has inline member bytes")
                }
            })
            .collect(),
        reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    });
    let directory = RelationalCheckpointDirectory {
        manifest: input.manifest,
        source: source.clone(),
    };
    let table = RuntimeSymbolTable::default();
    table.install_cold_pages(&directory).unwrap();
    (
        table,
        directory,
        source,
        runtime.services.symbols.snapshot(),
    )
}

#[test]
fn id_and_value_demands_load_pages_and_preserve_interning_identity_and_frontier() {
    let (table, directory, source, all) = dictionary();
    assert!(directory.manifest.symbols.len() > 1);
    assert!(table.snapshot().entries.is_empty());
    assert_eq!(source.reads.load(Ordering::SeqCst), 0);
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let first = all
        .entries
        .iter()
        .min_by_key(|(symbol, _)| *symbol)
        .unwrap();
    table.demand_symbol_ids(&execution, [first.0]).unwrap();
    assert!(table.with_read(|symbols| symbols.resolve(first.0) == Some(first.1.as_str())));
    assert!(source.reads.load(Ordering::SeqCst) < 3);
    assert!(table.snapshot().entries.len() < all.entries.len());
    let last = all
        .entries
        .iter()
        .max_by_key(|(symbol, _)| *symbol)
        .unwrap();
    assert!(table.with_read(|symbols| symbols.symbol(last.1.as_str()).is_none()));
    let before = source.reads.load(Ordering::SeqCst);
    table.demand_text(&execution, last.1.as_str()).unwrap();
    assert_eq!(
        table.with_write(|symbols| symbols.intern(last.1.as_str())),
        last.0
    );
    assert!(source.reads.load(Ordering::SeqCst) - before < 3);
    table
        .demand_text(&execution, "new-after-cold-directory")
        .unwrap();
    let issued = table.with_write(|symbols| symbols.intern("new-after-cold-directory"));
    assert!(
        issued.0 > last.0 .0,
        "partial dictionary admission must not reissue an old id"
    );
}

#[test]
fn a_damaged_symbol_page_is_a_sticky_typed_refusal() {
    let (table, _, source, all) = dictionary();
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let symbol = all.entries[0].0;
    source.corrupt.store(true, Ordering::SeqCst);
    let first = table.demand_symbol_ids(&execution, [symbol]).unwrap_err();
    assert!(matches!(&first, Failure::StorageUnavailable(failure)
        if matches!(failure.subject, RelationalStorageSubject::Symbols { .. }) && failure.cause == Cause::IntegrityMismatch));
    source.corrupt.store(false, Ordering::SeqCst);
    let reads = source.reads.load(Ordering::SeqCst);
    assert_eq!(
        table.demand_symbol_ids(&execution, [symbol]).unwrap_err(),
        first
    );
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
    assert!(table.snapshot().entries.is_empty());
}
