//! Resource proofs invoke actual metadata producers, not a reservation proxy.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::{EntityId, PartitionId};
use crate::storage::substrate::{
    EntityArena, EntityRecordKind, MetadataWriteDenial, RecordKind, SlotInit,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn fixed<T>() -> u64 {
    let header = std::alloc::Layout::new::<[usize; 2]>();
    header
        .extend(std::alloc::Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}
// Fixture-only public observation of the actual controller, never a private size guess.
fn controller_memory() -> u64 {
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut observed = None;
    request
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    let observed = observed.expect("the genuine controller callback was entered");
    assert!(observed > 0);
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed
}
fn init(version: u64) -> SlotInit<EntityRecordKind> {
    SlotInit {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        version_id: VersionId(version),
        extra: EntityExtra::default(),
    }
}
fn ordinary_arena() -> EntityArena {
    let mut arena = EntityArena::with_capacity(1);
    arena.write_reserved_slot(init(1), 0, 1).unwrap();
    arena
}
#[test]
fn metadata_controls_refuse_before_history_edits_with_typed_cause() {
    let bill = fixed::<VersionedEntityMetadata>();
    let controller = controller_memory();
    assert_ne!(
        controller, bill,
        "this oracle distinguishes controller and metadata denials"
    );
    let budget = SerialMemoryBudget::new(Some(controller.checked_add(bill).unwrap()));
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut arena = ordinary_arena();
    let before = rmp_serde::to_vec_named(&arena.metadata_history).unwrap();
    // Controller entry and the first metadata fit; only the retiring metadata copy is refused.
    let denied =
        arena.apply_extra_update_for_request(&request, 0, EntityExtra::default(), VersionId(2));
    assert!(
        matches!(denied, Err(Cause::PolicyMemoryExhausted { requested, .. }) if requested == bill)
    );
    assert_eq!(
        rmp_serde::to_vec_named(&arena.metadata_history).unwrap(),
        before
    );
    assert_eq!(arena.metadata_history[0][0].retired_at, None);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let zero = SerialMemoryBudget::new(Some(0));
    let zero_host = SerialRequest::from_memory(zero.clone(), cancel.token(), None);
    let mut empty = EntityArena::with_capacity(0);
    assert!(matches!(
        empty.write_reserved_slot_for_request(&ExecutionRequest::serial(&zero_host), init(1), 0, 1),
        Err(MetadataWriteDenial::Request(
            Cause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert_eq!(empty.slot_count(), 0);
    cancel.cancel();
    assert!(arena.retire_for_request(&request, 0, VersionId(3)).is_err());
    assert_eq!(
        rmp_serde::to_vec_named(&arena.metadata_history).unwrap(),
        before
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn metadata_controls_follow_actual_history_snapshots_and_refund() {
    let bill = fixed::<VersionedEntityMetadata>();
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut current = EntityArena::with_capacity(1);
    current
        .write_reserved_slot_for_request(&request, init(1), 0, 1)
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), bill);
    let old = current.clone();
    current
        .apply_extra_update_for_request(&request, 0, EntityExtra::default(), VersionId(2))
        .unwrap()
        .unwrap();
    assert_eq!(old.metadata_history[0].len(), 1);
    assert_eq!(old.metadata_history[0][0].retired_at, None);
    assert_eq!(
        current.metadata_history[0][0].retired_at,
        Some(VersionId(2))
    );
    assert_eq!(current.metadata_history[0][1].effective_at, VersionId(2));
    assert_eq!(budget.charged_memory_bytes(), 3 * bill);
    drop(old);
    assert_eq!(budget.charged_memory_bytes(), 2 * bill);
    let history = current.metadata_history[0].clone();
    drop(current);
    assert_eq!(budget.charged_memory_bytes(), 2 * bill);
    drop(history);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn moved_metadata_retains_only_its_fixed_bill_and_ordinary_clone_is_independent() {
    let bill = fixed::<VersionedEntityMetadata>();
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let request = ExecutionRequest::serial(&host);
    let actual = EntityRecordKind::metadata_for_create(
        KindId(1),
        1,
        VersionId(1),
        StorageAllocation::new(EntityExtra::default()),
    )
    .unwrap()
    .fund_for_request(&request)
    .unwrap();
    let actual = StorageAllocation::new(actual);
    let copy = actual.clone();
    assert_eq!(actual.id(), copy.id());
    drop(actual);
    let moved = StorageAllocation::unwrap_or_clone(copy);
    assert!(moved._funding.is_some());
    let ordinary = moved.clone();
    assert!(ordinary._funding.is_none());
    assert_eq!(
        budget.charged_memory_bytes(),
        bill,
        "only the fixed metadata bill survives the unique move"
    );
    drop(moved);
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(ordinary.effective_at, VersionId(1));
    println!(
        "metadata layouts entity={} relation={} entity_control={} relation_control={}",
        std::mem::size_of::<VersionedEntityMetadata>(),
        std::mem::size_of::<VersionedRelationMetadata>(),
        bill,
        fixed::<VersionedRelationMetadata>()
    );
}
#[test]
fn metadata_footer_preserves_legacy_named_and_compact_wire() {
    #[derive(serde::Serialize)]
    struct LegacyEntity<'a> {
        effective_at: VersionId,
        retired_at: Option<VersionId>,
        generation: u32,
        kind_id: KindId,
        lineage_id: Option<LineageId>,
        authoritative_aspect_state: Option<&'a AuthoritativeRecordAspectState>,
    }
    #[derive(serde::Serialize)]
    struct LegacyRelation<'a> {
        effective_at: VersionId,
        retired_at: Option<VersionId>,
        generation: u32,
        kind_id: KindId,
        endpoints: RelationEndpoints,
        authoritative_aspect_state: Option<&'a AuthoritativeRecordAspectState>,
    }
    let entity_bill = fixed::<VersionedEntityMetadata>();
    let relation_bill = fixed::<VersionedRelationMetadata>();
    let budget = SerialMemoryBudget::new(Some(64 * 1024));
    let cancel = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let request = ExecutionRequest::serial(&host);
    let entity = StorageAllocation::new(
        EntityRecordKind::metadata_for_create(
            KindId(1),
            3,
            VersionId(7),
            StorageAllocation::new(EntityExtra::default()),
        )
        .unwrap()
        .fund_for_request(&request)
        .unwrap(),
    );
    let endpoint = EntityId::new(PartitionId::main(), 0, 1);
    let endpoints = RelationEndpoints {
        source: endpoint,
        target: endpoint,
    };
    let extra = super::super::RelationExtra::from_unbudgeted_parts(Some(endpoints.clone()), None);
    let relation = StorageAllocation::new(
        crate::storage::substrate::RelationRecordKind::metadata_for_create(
            KindId(2),
            4,
            VersionId(8),
            StorageAllocation::new(extra),
        )
        .unwrap()
        .fund_for_request(&request)
        .unwrap(),
    );
    assert_eq!(budget.charged_memory_bytes(), entity_bill + relation_bill);
    let legacy = LegacyEntity {
        effective_at: entity.effective_at,
        retired_at: entity.retired_at,
        generation: entity.generation,
        kind_id: entity.kind_id,
        lineage_id: entity.lineage_id,
        authoritative_aspect_state: entity.authoritative_aspect_state(),
    };
    assert_eq!(
        rmp_serde::to_vec_named(&*entity).unwrap(),
        rmp_serde::to_vec_named(&legacy).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec(&*entity).unwrap(),
        rmp_serde::to_vec(&legacy).unwrap()
    );
    let legacy = LegacyRelation {
        effective_at: relation.effective_at,
        retired_at: relation.retired_at,
        generation: relation.generation,
        kind_id: relation.kind_id,
        endpoints: relation.endpoints.clone(),
        authoritative_aspect_state: relation.authoritative_aspect_state(),
    };
    assert_eq!(
        rmp_serde::to_vec_named(&*relation).unwrap(),
        rmp_serde::to_vec_named(&legacy).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec(&*relation).unwrap(),
        rmp_serde::to_vec(&legacy).unwrap()
    );
    assert_eq!(
        format!("{:?}", &*entity),
        format!("{:?}", entity.as_ref().clone())
    );
    drop((entity, relation));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
