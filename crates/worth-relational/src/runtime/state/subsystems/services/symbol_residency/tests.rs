use super::*;
mod page_admission_tests;
#[path = "routing_tests.rs"]
mod routing_tests;
use crate::durability::data::{
    RelationalCheckpointDirectory, RelationalCheckpointManifest, RelationalCheckpointMember,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::tests::support::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

mod region_admission_tests;

struct SymbolDictionaryWorld<'host, 'scope> {
    table: RuntimeSymbolTable,
    directory: RelationalCheckpointDirectory,
    source: Arc<PageSource>,
    all: crate::symbols::data::SymbolTableSnapshot,
    execution: worth_execution::ExecutionRequest<'host, 'scope>,
}
struct RegionPublicationWorld<'host, 'scope> {
    runtime: crate::runtime::RelationalRuntime,
    entity: crate::identity::data::EntityId,
    execution: worth_execution::ExecutionRequest<'host, 'scope>,
}
struct SymbolPageRefusalWorld<'host, 'scope> {
    dictionary: SymbolDictionaryWorld<'host, 'scope>,
    refused: worth_execution::ExecutionRequest<'host, 'scope>,
    budget: worth_execution::SerialMemoryBudget,
}
fn with_symbol_dictionary_world<R>(
    observe: impl for<'host, 'scope> FnOnce(SymbolDictionaryWorld<'host, 'scope>) -> R,
) -> R {
    let (table, directory, source, all) = dictionary();
    let host = crate::test_execution_policy::serial_request();
    observe(SymbolDictionaryWorld {
        table,
        directory,
        source,
        all,
        execution: worth_execution::ExecutionRequest::serial(&host),
    })
}
fn seeded_region(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> (
    crate::runtime::RelationalRuntime,
    crate::identity::data::EntityId,
) {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, name);
    (runtime, entity)
}
fn with_region_publication_world<R>(
    name: &str,
    observe: impl for<'host, 'scope> FnOnce(RegionPublicationWorld<'host, 'scope>) -> R,
) -> R {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (runtime, entity) = seeded_region(&execution, name);
    observe(RegionPublicationWorld {
        runtime,
        entity,
        execution,
    })
}
fn with_symbol_page_refusal_world<R>(
    observe: impl for<'host, 'scope> FnOnce(SymbolPageRefusalWorld<'host, 'scope>) -> R,
) -> R {
    let (table, directory, source, all) = dictionary();
    let host = crate::test_execution_policy::serial_request();
    let budget = worth_execution::SerialMemoryBudget::new(Some(0));
    let refused_host = crate::test_execution_policy::serial_request_with_memory(budget.clone());
    observe(SymbolPageRefusalWorld {
        dictionary: SymbolDictionaryWorld {
            table,
            directory,
            source,
            all,
            execution: worth_execution::ExecutionRequest::serial(&host),
        },
        refused: worth_execution::ExecutionRequest::serial(&refused_host),
        budget,
    })
}

#[derive(Debug)]
struct PageSource {
    members: BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
    reads: AtomicUsize,
    corrupt: AtomicBool,
    failed_member: Mutex<Option<RelationalDurableMemberLocator>>,
    failure: Mutex<Option<Cause>>,
}
impl RelationalDurableColdSource for PageSource {
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::medium::RelationalDurableReadPayload,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        request.check_live().map_err(|cause| {
            crate::durability::medium::RelationalDurableOwnedReadRefusal::Request(cause.into())
        })?;
        // This fixture's immutable member table is retained by its actual Source Arc.
        Ok(Box::new(()))
    }
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        crate::durability::medium::RelationalDurableMemberCustody::try_new_from_source_for_request(
            issuer,
            request,
            || Ok(()),
        )
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as R;
        if !original.is::<()>() {
            return Err(R::ForeignProtection);
        }
        if member.length > limit {
            return Err(R::Storage(Cause::LimitExceeded {
                length: member.length,
                limit,
            }));
        }
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.failed_member.lock().unwrap().as_ref() == Some(&member)
            || self.corrupt.load(Ordering::SeqCst)
        {
            return Err(R::Storage(Cause::Damaged));
        }
        if let Some(cause) = *self.failure.lock().unwrap() {
            return Err(R::Storage(cause));
        }
        let bytes = self
            .members
            .get(&member)
            .ok_or(R::Storage(Cause::MissingMember))?;
        if bytes.len() as u64 != member.length
            || crate::durability::log::native_checkpoint_codec::member_frame::locator(bytes.bytes())
                != member
        {
            return Err(R::Storage(Cause::IntegrityMismatch));
        }
        let mut output =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
                .map_err(R::Buffer)?;
        output.extend_from_slice(bytes.bytes()).map_err(R::Buffer)?;
        Ok(output.seal().map_err(R::Buffer)?.into_immutable_bytes())
    }

    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }

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
        if self.failed_member.lock().unwrap().as_ref() == Some(&locator) {
            return Err(Cause::Damaged);
        }
        if let Some(cause) = *self.failure.lock().unwrap() {
            return Err(cause);
        }
        let mut bytes = self
            .members
            .get(&locator)
            .ok_or(Cause::MissingMember)?
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
    checkpoint_dictionary(&execution, &runtime)
}

fn checkpoint_dictionary(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
) -> (
    RuntimeSymbolTable,
    RelationalCheckpointDirectory,
    Arc<PageSource>,
    crate::symbols::data::SymbolTableSnapshot,
) {
    let native = runtime
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
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
        failed_member: Mutex::new(None),
        failure: Mutex::new(None),
    });
    let directory = crate::durability::data::PreparedCheckpointDirectory {
        history_members: input
            .manifest
            .readmit_history_members(
                &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            )
            .unwrap(),
        manifest: input.manifest,
        source: source.clone(),
    }
    .readmit(execution)
    .unwrap();
    let table = RuntimeSymbolTable::default();
    table.install_cold_pages(&directory).unwrap();
    (
        table,
        directory,
        source,
        runtime.services.symbols.snapshot(),
    )
}

#[path = "placement_tests.rs"]
mod placement_tests;
