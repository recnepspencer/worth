//! Real Native publications preserve the selected public cache through movement and cancellation.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    mvcc::{
        CompanionBranchCellSlot, CompanionPreflightStop, PreparedPublicationCompanionEffect,
        PublicationCompanionPreflight, PublicationCompanionRegistration,
        RelationalPublicationCompanion,
    },
    runtime::{
        ManagedIndexBranchImage, ManagedIndexSourcePreparationDenial as Denial, RelationalRuntime,
    },
    transactions::{CommitExecutionDenialKind, TransactionCommitError},
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_publication::{batch_create, create_entity, memory_runtime},
};

use super::pinned_read_publication::snapshot_for_main;
type Payload = Arc<dyn std::any::Any + Send + Sync>;

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
    let before = snapshot_for_main(&runtime);
    let payload: Payload = Arc::new(10u64);
    let cache = runtime
        .read_truth()
        .retain_public_managed_index_cache(&execution, &before, &payload)
        .unwrap();
    let cancellation = CancellationSource::new();
    let denied = host.clone().with_cancellation(cancellation.token());
    *owner.cancel_after_preparation.lock().unwrap() = Some(cancellation);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            batch_create("cancelled-cache-publication"),
            execution.allocation_policy(),
        )
        .unwrap();
    let failure = transaction
        .commit(&ExecutionRequest::serial(&denied), &runtime)
        .unwrap_err();
    assert!(
        matches!(failure, TransactionCommitError::Execution { denial, .. }
        if matches!(denial.kind, CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled))),
        "{failure:?}"
    );
    assert_eq!(owner.prepared_cancellations.load(Ordering::SeqCst), 1);
    assert!(cache.is_at_head());
    let unchanged = snapshot_for_main(&runtime);
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
    let after = snapshot_for_main(&runtime);
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
    RelationalRuntime,
    Arc<Owner>,
    PublicationCompanionRegistration,
) {
    // No invariant Source program or audience exists in this schema.
    let runtime = memory_runtime();
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
    let before = snapshot_for_main(&runtime);
    let payload: Payload = Arc::new(10u64);
    let old = runtime
        .read_truth()
        .retain_public_managed_index_cache(&execution, &before, &payload)
        .unwrap();
    assert!(old.is_at_head());
    create_entity(&execution, &runtime, "public-after");
    let after = snapshot_for_main(&runtime);
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
