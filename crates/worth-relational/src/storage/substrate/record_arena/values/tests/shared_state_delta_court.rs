//! C2.5 Partition-wide exact set-difference courts:
//! 1. Unchanged slot A holding map M + new slot B holding M.clone().
//! 2. Cross-arena sharing: unchanged Entity slot A holding M + new Relation slot B holding M.clone().
//! 3. Older historical metadata sharing + replacement retaining a prior StoredValueText seal Arc.
use super::*;
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{
    EntityArena, RelationArena, SlotInit, StorageAllocationExtent, StorageAllocationObservation,
    StorageAllocationVisitor,
};
use std::collections::BTreeMap;

fn empty_partition() -> PartitionState {
    PartitionState {
        partition_id: PartitionId::main(),
        adjacency_policy: crate::config::data::AdjacencyPolicy {
            backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 8,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(4),
        relation_arena: RelationArena::with_capacity(4),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    }
}

fn sample_contracts() -> (AspectContract, AspectContract) {
    let field = |name: &str| FieldKey::new(name).unwrap();
    let struct_contract = AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(21),
        AspectContractRevision(1),
        StructAspectShape::new(["title", "notes"].map(|name| {
            FieldDeclaration::new(
                field(name),
                ScalarAspectType::String,
                FieldRequirement::Optional,
                AbsenceLaw::Optional,
                AspectEvolutionPolicy::AdditiveFieldsAllowed,
            )
            .unwrap()
        }))
        .unwrap(),
    );
    let scalar_contract = AspectContract::scalar(
        AspectKey::new("label").unwrap(),
        AspectIdentity(22),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    (struct_contract, scalar_contract)
}

fn build_state(
    struct_contract: &AspectContract,
    scalar_contract: &AspectContract,
    title: &str,
    stored_notes: StoredValueText,
    label: &str,
) -> AuthoritativeRecordAspectState {
    let field = |name: &str| FieldKey::new(name).unwrap();
    let TransitionOutcome::Success(struct_val) = validate_aspect_value(
        struct_contract,
        ContractValidationInput::Struct(
            StructAspectValue::new([
                (
                    field("title"),
                    AspectValue::String(InternedString::Raw(title.into())),
                ),
                (
                    field("notes"),
                    AspectValue::String(InternedString::Stored(stored_notes)),
                ),
            ])
            .unwrap(),
        ),
    ) else {
        panic!("admitted struct")
    };
    let TransitionOutcome::Success(scalar_val) = validate_aspect_value(
        scalar_contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(label.into()))),
    ) else {
        panic!("admitted scalar")
    };
    let TransitionOutcome::Success(admitted) =
        admit_authoritative_record_aspect_state([struct_val, scalar_val])
    else {
        panic!("admitted state")
    };
    admitted.into_parts().into_parts().0
}

fn value_backing_bytes(state: &AuthoritativeRecordAspectState) -> u64 {
    #[derive(Default)]
    struct Seen(BTreeMap<u64, u64>);
    impl StorageAllocationVisitor for Seen {
        fn visit(&mut self, obs: StorageAllocationObservation) -> bool {
            assert_eq!(obs.extent, StorageAllocationExtent::Checked);
            self.0.insert(obs.id, obs.bytes).is_none()
        }
    }
    let mut seen = Seen::default();
    crate::storage::substrate::visit_foundational_value_backing(state, false, &mut seen);
    seen.0.values().sum()
}

fn assert_partition_delta_exact(before: &PartitionState, after: &PartitionState) {
    let before_inv = before.allocation_inventory().unwrap();
    let after_inv = after.allocation_inventory().unwrap();
    let updated = after.allocation_inventory_since(before, before_inv).unwrap();
    assert_eq!(
        updated, after_inv,
        "incremental partition inventory delta must match full partition inventory exactly"
    );
    let introduced = after.new_authoritative_allocation_bytes_since(before).unwrap();
    let removed = before.new_authoritative_allocation_bytes_since(after).unwrap();
    assert_eq!(
        before_inv.authoritative_bytes - removed + introduced,
        after_inv.authoritative_bytes,
        "exact authoritative set-difference identity must hold across the partition"
    );
}

fn write_entity(
    partition: &mut PartitionState,
    slot: usize,
    version: u64,
    state: AuthoritativeRecordAspectState,
) {
    partition
        .entity_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                version_id: VersionId(version),
                extra: EntityExtra::from_unbudgeted_parts(None, None, Some(state)),
            },
            slot,
            1,
        )
        .unwrap();
}

fn write_relation(
    partition: &mut PartitionState,
    slot: usize,
    version: u64,
    state: AuthoritativeRecordAspectState,
) {
    partition
        .relation_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                version_id: VersionId(version),
                extra: RelationExtra::from_unbudgeted_parts(
                    Some(RelationEndpoints {
                        source: EntityId::new(PartitionId::main(), 0, 1),
                        target: EntityId::new(PartitionId::main(), 0, 1),
                    }),
                    Some(state),
                ),
            },
            slot,
            1,
        )
        .unwrap();
}

#[test]
fn unchanged_slot_a_and_new_slot_b_sharing_map_count_shared_backing_once() {
    let (sc, lc) = sample_contracts();
    let shared_map = build_state(
        &sc,
        &lc,
        "shared-title-a",
        StoredValueText::new("shared-stored-notes-a".into()),
        "shared-label-a",
    );
    let shared_bytes = value_backing_bytes(&shared_map);
    assert!(shared_bytes > 0);

    let mut before = empty_partition();
    write_entity(&mut before, 0, 1, shared_map.clone());

    // Successor 1 adds a distinct Extra at slot 1 holding `shared_map.clone()` while slot 0 is untouched.
    let mut after_shared = before.clone();
    write_entity(&mut after_shared, 1, 2, shared_map);

    // Successor 2 adds an identically-sized but independently allocated map at slot 1.
    let independent_map = build_state(
        &sc,
        &lc,
        "shared-title-a",
        StoredValueText::new("shared-stored-notes-a".into()),
        "shared-label-a",
    );
    assert_eq!(value_backing_bytes(&independent_map), shared_bytes);
    let mut after_independent = before.clone();
    write_entity(&mut after_independent, 1, 2, independent_map);

    assert_partition_delta_exact(&before, &after_shared);
    assert_partition_delta_exact(&before, &after_independent);
    let introduced_shared = after_shared
        .new_authoritative_allocation_bytes_since(&before)
        .unwrap();
    let introduced_independent = after_independent
        .new_authoritative_allocation_bytes_since(&before)
        .unwrap();
    assert_eq!(
        introduced_independent - introduced_shared,
        shared_bytes,
        "sharing slot A's map in new slot B must save the exact shared_map backing bytes in delta"
    );
    let full_shared = after_shared.allocation_inventory().unwrap().authoritative_bytes;
    let full_independent = after_independent
        .allocation_inventory()
        .unwrap()
        .authoritative_bytes;
    assert_eq!(
        full_independent - full_shared,
        shared_bytes,
        "sharing slot A's map in new slot B must save the exact shared_map backing bytes in full inventory"
    );
}

#[test]
fn cross_arena_entity_and_relation_sharing_map_unions_full_inventory_and_delta() {
    let (sc, lc) = sample_contracts();
    let shared_map = build_state(
        &sc,
        &lc,
        "cross-arena-title-payload-for-deduplication",
        StoredValueText::new("cross-arena-stored-notes-payload-for-deduplication".into()),
        "cross-arena-label-payload",
    );
    let shared_bytes = value_backing_bytes(&shared_map);
    assert!(shared_bytes > 0);

    let mut before = empty_partition();
    write_entity(&mut before, 0, 1, shared_map.clone());

    // Successor leaves entity_arena untouched and adds a relation sharing `shared_map.clone()`.
    let mut after = before.clone();
    write_relation(&mut after, 0, 2, shared_map);

    // Full partition inventory must deduplicate across entity_arena and relation_arena.
    let entity_only = after
        .entity_arena
        .allocation_inventory()
        .unwrap()
        .authoritative_bytes;
    let relation_only = after
        .relation_arena
        .allocation_inventory()
        .unwrap()
        .authoritative_bytes;
    let full_partition = after.allocation_inventory().unwrap().authoritative_bytes;
    assert_eq!(
        full_partition,
        entity_only + relation_only - shared_bytes,
        "full PartitionState::allocation_inventory must deduplicate backing shared across entity and relation arenas"
    );

    // Incremental delta must match exact full inventory and save exact shared_map bytes
    // compared to an identically-sized independently allocated relation map.
    assert_partition_delta_exact(&before, &after);
    let independent_map = build_state(
        &sc,
        &lc,
        "cross-arena-title-payload-for-deduplication",
        StoredValueText::new("cross-arena-stored-notes-payload-for-deduplication".into()),
        "cross-arena-label-payload",
    );
    assert_eq!(value_backing_bytes(&independent_map), shared_bytes);
    let mut after_independent = before.clone();
    write_relation(&mut after_independent, 0, 2, independent_map);
    assert_partition_delta_exact(&before, &after_independent);
    let introduced_shared = after.new_authoritative_allocation_bytes_since(&before).unwrap();
    let introduced_independent = after_independent
        .new_authoritative_allocation_bytes_since(&before)
        .unwrap();
    assert_eq!(
        introduced_independent - introduced_shared,
        shared_bytes,
        "cross-arena delta must charge shared_map zero additional bytes"
    );
}

#[test]
fn older_metadata_history_and_cloned_stored_text_seal_are_deduplicated_in_deltas() {
    let (sc, lc) = sample_contracts();
    let original_stored = StoredValueText::new("retained-seal-payload-text".into());
    let state_v1 = build_state(&sc, &lc, "v1-title", original_stored.clone(), "v1-label");
    let state_v2 = build_state(
        &sc,
        &lc,
        "v2-completely-different-title",
        StoredValueText::new("v2-fresh-stored-text".into()),
        "v2-different-label",
    );
    // v3 reuses a fresh StoredValueText clone (new Box + String buffer, SAME Arc<OnceLock<CodecSeal>>
    // as v1, which now lives ONLY in older metadata_history[0] of p2!) plus an unchanged v1 aspect map.
    let state_v3_cloned_seal = build_state(
        &sc,
        &lc,
        "v3-fresh-title",
        original_stored.clone(),
        "v3-fresh-label",
    );

    let mut p1 = empty_partition();
    p1.entity_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                version_id: VersionId(1),
                extra: EntityExtra::from_unbudgeted_parts(None, None, Some(state_v1.clone())),
            },
            0,
            1,
        )
        .unwrap();

    // p2 supersedes current extra[0] with state_v2; state_v1 + original_stored.seal remain
    // ONLY in older metadata_history[0].
    let mut p2 = p1.clone();
    p2.entity_arena
        .apply_extra_update(
            0,
            EntityExtra::from_unbudgeted_parts(None, None, Some(state_v2)),
            VersionId(2),
        )
        .unwrap();
    assert_partition_delta_exact(&p1, &p2);

    // p3 writes a new extra referencing state_v1.clone() (present ONLY in p2's older history!).
    let mut p3_reused_history = p2.clone();
    p3_reused_history
        .entity_arena
        .apply_extra_update(
            0,
            EntityExtra::from_unbudgeted_parts(None, None, Some(state_v1.clone())),
            VersionId(3),
        )
        .unwrap();
    assert_partition_delta_exact(&p2, &p3_reused_history);

    // p4 replaces slot 0 with a cloned StoredValueText sharing original_stored's seal Arc.
    let mut p4_cloned_seal = p2.clone();
    p4_cloned_seal
        .entity_arena
        .apply_extra_update(
            0,
            EntityExtra::from_unbudgeted_parts(None, None, Some(state_v3_cloned_seal)),
            VersionId(3),
        )
        .unwrap();
    assert_partition_delta_exact(&p2, &p4_cloned_seal);

    // Also verify direct field/scalar replacement on a slot whose current extra holds original_stored.
    let mut p5_direct_seal_replace = p1.clone();
    let replacement = build_state(&sc, &lc, "v1-title", original_stored, "v1-label");
    p5_direct_seal_replace
        .entity_arena
        .apply_extra_update(
            0,
            EntityExtra::from_unbudgeted_parts(None, None, Some(replacement)),
            VersionId(2),
        )
        .unwrap();
    assert_partition_delta_exact(&p1, &p5_direct_seal_replace);
}
