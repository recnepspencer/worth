//! Pure partition fixture and observation-ID set-difference helpers for
//! incremental publication locality courts (no runtime/execution authority).
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{
    EntityArena, EntityExtra, RelationArena, SealedAllocationWatermarks, SlotInit,
    StorageAllocationExtent, StorageAllocationObservation, StorageAllocationVisitor,
};
use std::collections::BTreeMap;
use worth_foundational::facade::{
    admit_authoritative_record_aspect_state, validate_aspect_value, AbsenceLaw, AspectContract,
    AspectContractRevision, AspectEvolutionPolicy, AspectIdentity, AspectKey, AspectValue,
    ContractValidationInput, FieldDeclaration, FieldKey, FieldRequirement, InternedString,
    ScalarAspectType, StoredValueText, StructAspectShape, StructAspectValue,
};
use worth_proof::TransitionOutcome;

pub(super) fn stored_label_contract() -> AspectContract {
    AspectContract::scalar(
        AspectKey::new("label").unwrap(),
        AspectIdentity(91),
        AspectContractRevision(1),
        ScalarAspectType::String,
    )
}

pub(super) fn struct_payload_contract() -> AspectContract {
    let shape = StructAspectShape::new([
        FieldDeclaration::new(
            FieldKey::new("count").unwrap(),
            ScalarAspectType::UInt64,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap(),
        FieldDeclaration::new(
            FieldKey::new("note").unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap(),
    ])
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("payload").unwrap(),
        AspectIdentity(92),
        AspectContractRevision(1),
        shape,
    )
}

pub(super) fn make_one_field_struct(count: u64) -> StructAspectValue {
    StructAspectValue::from_sorted_fields(
        vec![(FieldKey::new("count").unwrap(), AspectValue::UInt64(count))].into_boxed_slice(),
    )
    .unwrap()
}

pub(super) fn make_struct_extra(
    contract: &AspectContract,
    value: StructAspectValue,
) -> EntityExtra {
    let TransitionOutcome::Success(val) =
        validate_aspect_value(contract, ContractValidationInput::Struct(value))
    else {
        panic!("admitted struct")
    };
    let TransitionOutcome::Success(admitted) = admit_authoritative_record_aspect_state([val])
    else {
        panic!("admitted state")
    };
    EntityExtra::from_unbudgeted_parts(None, None, Some(admitted.into_parts().into_parts().0))
}

pub(super) fn seed_struct_partition(
    contract: &AspectContract,
    population: usize,
    history_depth: u64,
) -> (PartitionState, StructAspectValue) {
    let mut base = PartitionState {
        partition_id: PartitionId(1),
        adjacency_policy: crate::config::data::AdjacencyPolicy {
            backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(population),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let mut slot1_struct = None;
    for slot in 0..population {
        let seed = make_one_field_struct((slot as u64) * 1000);
        if slot == 1 {
            slot1_struct = Some(seed.clone());
        }
        base.entity_arena
            .write_reserved_slot(
                SlotInit {
                    partition_id: base.partition_id,
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: make_struct_extra(contract, seed),
                },
                slot,
                1,
            )
            .unwrap();
        for step in 1..history_depth {
            let step_val = make_one_field_struct((slot as u64) * 1000 + step);
            if slot == 1 {
                slot1_struct = Some(step_val.clone());
            }
            base.entity_arena
                .apply_extra_update(
                    slot,
                    make_struct_extra(contract, step_val),
                    VersionId(1 + step),
                )
                .unwrap();
        }
    }
    (base, slot1_struct.unwrap())
}

pub(super) fn make_stored_extra(contract: &AspectContract, text: StoredValueText) -> EntityExtra {
    let TransitionOutcome::Success(val) = validate_aspect_value(
        contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Stored(text))),
    ) else {
        panic!("admitted scalar")
    };
    let TransitionOutcome::Success(admitted) = admit_authoritative_record_aspect_state([val])
    else {
        panic!("admitted state")
    };
    EntityExtra::from_unbudgeted_parts(None, None, Some(admitted.into_parts().into_parts().0))
}

pub(super) fn seed_stored_partition(
    contract: &AspectContract,
    population: usize,
    history_depth: u64,
) -> (PartitionState, StoredValueText) {
    let mut base = PartitionState {
        partition_id: PartitionId(1),
        adjacency_policy: crate::config::data::AdjacencyPolicy {
            backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(population),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let mut slot1_stored = None;
    for slot in 0..population {
        let seed = StoredValueText::new(format!("seed-{slot}-0"));
        if slot == 1 {
            slot1_stored = Some(seed.clone());
        }
        base.entity_arena
            .write_reserved_slot(
                SlotInit {
                    partition_id: base.partition_id,
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: make_stored_extra(contract, seed),
                },
                slot,
                1,
            )
            .unwrap();
        for step in 1..history_depth {
            let step_text = StoredValueText::new(format!("seed-{slot}-{step}"));
            if slot == 1 {
                slot1_stored = Some(step_text.clone());
            }
            base.entity_arena
                .apply_extra_update(
                    slot,
                    make_stored_extra(contract, step_text),
                    VersionId(1 + step),
                )
                .unwrap();
        }
    }
    (base, slot1_stored.unwrap())
}

fn collect_all(partition: &PartitionState) -> BTreeMap<u64, u64> {
    struct All(BTreeMap<u64, u64>);
    impl StorageAllocationVisitor for All {
        fn visit(&mut self, obs: StorageAllocationObservation) -> bool {
            assert_eq!(obs.extent, StorageAllocationExtent::Checked);
            self.0.insert(obs.id, obs.bytes).is_none()
        }
    }
    let mut all = All(BTreeMap::new());
    partition.visit_authoritative_allocations(false, &mut all);
    all.0
}

pub(super) fn assert_directed_ids(
    prev: &PartitionState,
    next: &PartitionState,
    watermarks: SealedAllocationWatermarks,
) -> u64 {
    struct Set(BTreeMap<u64, u64>);
    impl StorageAllocationVisitor for Set {
        fn visit(&mut self, obs: StorageAllocationObservation) -> bool {
            assert_eq!(obs.extent, StorageAllocationExtent::Checked);
            self.0.insert(obs.id, obs.bytes).is_none()
        }
    }
    let prev_all = collect_all(prev);
    let next_all = collect_all(next);
    let exp_fwd: BTreeMap<_, _> = next_all
        .iter()
        .filter(|(id, _)| !prev_all.contains_key(id))
        .map(|(k, v)| (*k, *v))
        .collect();
    let exp_rev: BTreeMap<_, _> = prev_all
        .iter()
        .filter(|(id, _)| !next_all.contains_key(id))
        .map(|(k, v)| (*k, *v))
        .collect();
    let mut fwd = Set(BTreeMap::new());
    next.visit_new_authoritative_allocations_since_with_watermarks(prev, watermarks, &mut fwd);
    let mut rev = Set(BTreeMap::new());
    prev.visit_new_authoritative_allocations_since_with_watermarks(
        next,
        SealedAllocationWatermarks::default(),
        &mut rev,
    );
    assert_eq!(
        fwd.0, exp_fwd,
        "forward emitted IDs must equal A(next) \\ A(prev)"
    );
    assert_eq!(
        rev.0, exp_rev,
        "reverse emitted IDs must equal A(prev) \\ A(next)"
    );
    exp_fwd.values().sum()
}
