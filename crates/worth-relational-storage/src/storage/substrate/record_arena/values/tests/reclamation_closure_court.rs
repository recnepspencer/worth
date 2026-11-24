//! C2.5 Exact internal-refcount graph closure court for retiring roots:
//! multiple internal owning edges (`extra` + `metadata_history`, multiple slots,
//! cross-arena Entity+Relation sharing, and shared StoredValueText seals) must
//! reclaim 100% of exclusively owned backing while excluding any allocation
//! still held by an external owner.
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

fn contracts() -> (AspectContract, AspectContract) {
    let field = |name: &str| FieldKey::new(name).unwrap();
    let sc = AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(31),
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
    let lc = AspectContract::scalar(
        AspectKey::new("label").unwrap(),
        AspectIdentity(32),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    (sc, lc)
}

fn build_state(
    sc: &AspectContract,
    lc: &AspectContract,
    title: &str,
    notes: StoredValueText,
    label: &str,
) -> AuthoritativeRecordAspectState {
    let field = |name: &str| FieldKey::new(name).unwrap();
    let TransitionOutcome::Success(sv) = validate_aspect_value(
        sc,
        ContractValidationInput::Struct(
            StructAspectValue::new([
                (
                    field("title"),
                    AspectValue::String(InternedString::Raw(title.into())),
                ),
                (
                    field("notes"),
                    AspectValue::String(InternedString::Stored(notes)),
                ),
            ])
            .unwrap(),
        ),
    ) else {
        panic!("admitted struct")
    };
    let TransitionOutcome::Success(lv) = validate_aspect_value(
        lc,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(label.into()))),
    ) else {
        panic!("admitted scalar")
    };
    let TransitionOutcome::Success(admitted) = admit_authoritative_record_aspect_state([sv, lv])
    else {
        panic!("admitted state")
    };
    admitted.into_parts().into_parts().0
}

fn value_backing_ids(state: &AuthoritativeRecordAspectState) -> BTreeMap<u64, u64> {
    #[derive(Default)]
    struct Collect(BTreeMap<u64, u64>);
    impl StorageAllocationVisitor for Collect {
        fn visit(&mut self, obs: StorageAllocationObservation) -> bool {
            assert_eq!(obs.extent, StorageAllocationExtent::Checked);
            self.0.insert(obs.id, obs.bytes).is_none()
        }
    }
    let mut collect = Collect::default();
    crate::storage::substrate::visit_foundational_value_backing(state, false, &mut collect);
    collect.0
}

fn full_authoritative_ids(partition: &PartitionState) -> BTreeMap<u64, u64> {
    #[derive(Default)]
    struct Collect(BTreeMap<u64, u64>);
    impl StorageAllocationVisitor for Collect {
        fn visit(&mut self, obs: StorageAllocationObservation) -> bool {
            assert_eq!(obs.extent, StorageAllocationExtent::Checked);
            self.0.insert(obs.id, obs.bytes).is_none()
        }
    }
    let mut collect = Collect::default();
    partition.visit_authoritative_allocations(false, &mut collect);
    collect.0
}

#[test]
fn multi_edge_reclaimable_bytes_match_independent_full_inventory_oracle() {
    let (sc, lc) = contracts();
    let shared_stored = StoredValueText::new("reclaim-closure-shared-seal".into());
    let shared_map = build_state(
        &sc,
        &lc,
        "multi-edge-title",
        shared_stored.clone(),
        "multi-edge-label",
    );
    let cloned_seal_map = build_state(
        &sc,
        &lc,
        "cloned-seal-title",
        shared_stored,
        "cloned-seal-label",
    );
    let shared_map_ids = value_backing_ids(&shared_map);
    let shared_map_bytes: u64 = shared_map_ids.values().sum();
    assert!(shared_map_bytes > 0);

    // 1. Single slot where both `extra[0]` and `metadata_history[0][0]` hold
    // the SAME `StorageAllocation<EntityExtra>` (strong_count == 2 internally).
    let mut single = empty_partition();
    single
        .entity_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                version_id: VersionId(1),
                extra: EntityExtra::from_unbudgeted_parts(None, None, Some(shared_map.clone())),
            },
            0,
            1,
        )
        .unwrap();
    // Hold an external clone of `shared_map` while checking single-slot closure:
    // `EntityExtra` itself (strong_count == 2 inside `single`) MUST be reclaimed,
    // while `shared_map` (held externally) MUST be excluded.
    let single_oracle = full_authoritative_ids(&single);
    let single_total: u64 = single_oracle.values().sum();
    assert_eq!(
        single_total,
        single.allocation_inventory().unwrap().authoritative_bytes
    );
    // `shared_map` and `cloned_seal_map` still share the StoredValueText seal externally.
    let external_ids: std::collections::BTreeSet<u64> = shared_map_ids
        .keys()
        .copied()
        .chain(value_backing_ids(&cloned_seal_map).into_keys())
        .collect();
    let expected_single_without_external: u64 = single_oracle
        .iter()
        .filter(|(id, _)| !external_ids.contains(id))
        .map(|(_, bytes)| *bytes)
        .sum();
    assert_eq!(
        single.reclaimable_unique_authoritative_bytes().unwrap(),
        expected_single_without_external,
        "current+history internal edges (strong_count == 2) must reclaim the Extra carrier while excluding externally held map backing"
    );

    // 2. Multi-slot + cross-arena partition owning ALL edges of `shared_map` and
    // `cloned_seal_map` (2 Entity slots + 1 Relation slot + metadata_history).
    let mut multi = empty_partition();
    multi
        .entity_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                version_id: VersionId(1),
                extra: EntityExtra::from_unbudgeted_parts(None, None, Some(shared_map.clone())),
            },
            0,
            1,
        )
        .unwrap();
    multi
        .entity_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                version_id: VersionId(2),
                extra: EntityExtra::from_unbudgeted_parts(None, None, Some(cloned_seal_map)),
            },
            1,
            1,
        )
        .unwrap();
    multi
        .relation_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                version_id: VersionId(3),
                extra: RelationExtra::from_unbudgeted_parts(
                    Some(RelationEndpoints {
                        source: EntityId::new(PartitionId::main(), 0, 1),
                        target: EntityId::new(PartitionId::main(), 1, 1),
                    }),
                    Some(shared_map.clone()),
                ),
            },
            0,
            1,
        )
        .unwrap();
    drop(single);

    let multi_oracle = full_authoritative_ids(&multi);
    let multi_total: u64 = multi_oracle.values().sum();
    assert_eq!(
        multi_total,
        multi.allocation_inventory().unwrap().authoritative_bytes
    );

    // While `shared_map` is still held externally in this local variable,
    // ONLY `shared_map`'s value-backing IDs are excluded from reclaimable bytes.
    assert_eq!(
        multi.reclaimable_unique_authoritative_bytes().unwrap(),
        multi_total - shared_map_bytes,
        "external handle to shared_map must exclude exactly shared_map's value-backing bytes"
    );

    // Drop the last external handle: `multi` is now the SOLE owner of every
    // carrier, every multi-edge map node, and the shared StoredValueText seal.
    drop(shared_map);
    assert_eq!(
        multi.reclaimable_unique_authoritative_bytes().unwrap(),
        multi_total,
        "exclusively owned multi-edge partition must reclaim 100% of its independent full-inventory authoritative bytes"
    );
}
