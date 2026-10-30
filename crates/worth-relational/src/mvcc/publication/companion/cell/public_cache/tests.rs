//! Real Native publications, selected predecessors and final opaque cleanup.
use super::*;
use crate::{
    mvcc::{
        PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
        RelationalPublicationCompanion,
    },
    tests::support::{create_entity, snapshot_for_owner_identity, test_schema_registry},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[derive(Default)]
struct Owner {
    slot: Mutex<Option<CompanionBranchCellSlot<ManagedIndexBranchImage>>>,
    cancel_after_preparation: Mutex<Option<CancellationSource>>,
    prepared_cancellations: AtomicUsize,
}
impl std::fmt::Debug for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ConservativePublicCacheOwner")
    }
}
impl RelationalPublicationCompanion for Owner {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let mark = context.mint_selected_branch_cell(Arc::new(()))?;
        let mark = mark.reserve_preflight(context)?;
        let mut slot = self.slot.lock().unwrap();
        let managed = if let Some(cell) = slot.as_ref().and_then(|slot| slot.admitted()) {
            cell.reserve_preflight(context)?
        } else {
            let cell = context.mint_managed_index_source_cell()?;
            *slot = Some(cell.lookup_slot());
            cell.reserve_preflight(context)?
        };
        drop(slot);
        let next = context.prepare_managed_source_image(&managed)?;
        if let Some(cancellation) = self.cancel_after_preparation.lock().unwrap().take() {
            self.prepared_cancellations.fetch_add(1, Ordering::SeqCst);
            cancellation.cancel();
            context.checkpoint()?;
        }
        context.seal_managed_source_replacement(mark, Arc::new(()), managed, next)
    }
}

#[test]
fn public_cache_cancelled_preflight_preserves_old_material_and_successful_twin_is_cold() {
    let (runtime, owner, registration) = install();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    create_entity(&execution, &runtime, "public-cancel-before");
    let before = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let payload: Payload = Arc::new(10u64);
    let cache = runtime
        .read_truth()
        .retain_public_managed_index_cache(&execution, &before, &payload)
        .unwrap();
    let cancellation = CancellationSource::new();
    let denied = host.clone().with_cancellation(cancellation.token());
    *owner.cancel_after_preparation.lock().unwrap() = Some(cancellation);
    let mut transaction = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            crate::transactions::data::WorkerIntentBatch::new("public-cancelled-preflight").push(
                crate::transactions::data::MutationIntent::Create(
                    crate::transactions::data::CreateIntent::Entity(
                        crate::transactions::data::EntitySpec {
                            partition_id: crate::identity::data::PartitionId::main(),
                            kind_id: crate::identity::data::KindId(1),
                            client_key: crate::symbols::data::ClientKey::raw(
                                "cancelled-cache-publication",
                            ),
                            fields: crate::tests::support::single_string_aspect_field_patch(
                                crate::tests::support::aspect_key("name"),
                                crate::tests::support::field_key("name"),
                                "cancelled",
                            ),
                        },
                    ),
                ),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let failure = transaction
        .commit(&ExecutionRequest::serial(&denied), &runtime)
        .unwrap_err();
    assert!(
        matches!(failure, crate::transactions::data::TransactionCommitError::Execution { denial, .. }
        if matches!(denial.kind, crate::transactions::data::CommitExecutionDenialKind::Cause(crate::execution::RelationalExecutionDenialCause::Cancelled))),
        "{failure:?}"
    );
    assert_eq!(owner.prepared_cancellations.load(Ordering::SeqCst), 1);
    assert!(cache.is_at_head());
    let unchanged = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    assert_eq!(
        *runtime
            .read_truth()
            .public_managed_index_cache_payload(&execution, &unchanged, &cache)
            .unwrap()
            .downcast_ref::<u64>()
            .unwrap(),
        10
    );
    create_entity(&execution, &runtime, "public-cancel-retry");
    let after = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    assert!(!cache.is_at_head());
    assert!(matches!(
        runtime
            .read_truth()
            .public_managed_index_cache_payload(&execution, &after, &cache),
        Err(Denial::RootUnavailable)
    ));
    drop(cache.retire());
    drop((cache, registration, owner, runtime));
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
fn install() -> (
    crate::runtime::RelationalRuntime,
    Arc<Owner>,
    crate::mvcc::PublicationCompanionRegistration,
) {
    // No invariant Source program or audience exists in this schema.
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .build()
        .unwrap();
    let owner = Arc::new(Owner::default());
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(owner.clone())
        .unwrap();
    (runtime, owner, registration)
}

#[test]
fn public_cache_native_successor_is_cold_and_selected_predecessor_keeps_its_own_slot() {
    let (runtime, owner, registration) = install();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    create_entity(&execution, &runtime, "public-before");
    let before = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let payload: Payload = Arc::new(10u64);
    let old = runtime
        .read_truth()
        .retain_public_managed_index_cache(&execution, &before, &payload)
        .unwrap();
    assert!(old.is_at_head());
    create_entity(&execution, &runtime, "public-after");
    let after = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    assert!(!old.is_at_head());
    assert_eq!(
        *runtime
            .read_truth()
            .public_managed_index_cache_payload(&execution, &before, &old)
            .unwrap()
            .downcast_ref::<u64>()
            .unwrap(),
        10
    );
    assert!(matches!(
        runtime
            .read_truth()
            .public_managed_index_cache_payload(&execution, &after, &old),
        Err(Denial::RootUnavailable)
    ));
    let payload: Payload = Arc::new(20u64);
    let new = runtime
        .read_truth()
        .retain_public_managed_index_cache(&execution, &after, &payload)
        .unwrap();
    assert!(matches!(
        runtime
            .read_truth()
            .public_managed_index_cache_payload(&execution, &before, &new),
        Err(Denial::RootUnavailable)
    ));
    drop(old.retire());
    assert_eq!(
        *runtime
            .read_truth()
            .public_managed_index_cache_payload(&execution, &after, &new)
            .unwrap()
            .downcast_ref::<u64>()
            .unwrap(),
        20,
        "retiring an old image's same ordinal cannot clear its successor"
    );
    drop(new.retire());
    assert!(!new.is_at_head());
    assert!(matches!(
        runtime
            .read_truth()
            .public_managed_index_cache_payload(&execution, &after, &new),
        Err(Denial::RootUnavailable)
    ));
    drop((old, new, registration, owner, runtime));
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}

struct Probe {
    core: Weak<CompanionBranchCellCore>,
    image: Weak<ManagedIndexBranchImage>,
    port: crate::mvcc::PublicationCompanionRegistrationPort,
    drops: Arc<AtomicUsize>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        assert!(self.core.upgrade().unwrap().current.try_lock().is_ok());
        assert!(self
            .image
            .upgrade()
            .unwrap()
            .public_caches
            .try_lock()
            .is_ok());
        let _registration = self
            .port
            .begin_required_registration()
            .expect("last opaque Drop occurs beyond the genuine Native registration epoch");
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn public_cache_retirement_defers_opaque_drop_and_releases_original_backing() {
    let (runtime, owner, registration) = install();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    create_entity(&execution, &runtime, "public-retirement");
    let snapshot = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let cell = owner
        .slot
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .admitted()
        .unwrap();
    let image = Arc::clone(&cell.core.current.lock().unwrap().payload)
        .downcast::<ManagedIndexBranchImage>()
        .unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let payload: Payload = Arc::new(Probe {
        core: Arc::downgrade(&cell.core),
        image: Arc::downgrade(&image),
        port: runtime.publication_companion_port(),
        drops: Arc::clone(&drops),
    });
    let cache = runtime
        .read_truth()
        .retain_public_managed_index_cache(&execution, &snapshot, &payload)
        .unwrap();
    drop(payload);
    let publication = runtime.publication_binding();
    let epoch = publication.companion_epoch().unwrap();
    let current = cell.core.current.lock().unwrap();
    let retired = cache.retire();
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop((current, epoch));
    drop(cache);
    drop(retired);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(image.public_caches.lock().unwrap().entries.is_empty());
    drop((image, cell, registration, owner, runtime));
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
