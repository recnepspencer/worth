//! C2.5 Sharing Court: distinct Extras sharing one aspect map contribute the
//! shared backing once, and rows sum to the allocator-observed physical union.
use super::*;
use crate::storage::substrate::{StorageAllocationExtent, StorageAllocationOverflow};
use std::collections::BTreeMap;

#[derive(Default)]
struct UniqueRows(BTreeMap<u64, u64>);
impl StorageAllocationVisitor for UniqueRows {
    fn visit(&mut self, observation: StorageAllocationObservation) -> bool {
        assert_eq!(observation.extent, StorageAllocationExtent::Checked);
        if let Some(previous) = self.0.insert(observation.id, observation.bytes) {
            assert_eq!(previous, observation.bytes);
            false
        } else {
            true
        }
    }
}

#[test]
fn extras_sharing_one_aspect_map_contribute_shared_backing_once_matching_allocator_union() {
    let field = |name: &str| FieldKey::new(name).unwrap();
    let struct_contract = AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(7),
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
        AspectIdentity(8),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );

    // Measure the exact allocator-observed physical backing of the shared map's
    // value backing (excluding SchemaContract allocations owned by RootSchemaAuthority)
    // plus the two distinct StorageAllocation<EntityExtra> carriers.
    let (state, shared_value_backing_bytes) = {
        let mut contracts_only = UniqueRows::default();
        let TransitionOutcome::Success(struct_val) = validate_aspect_value(
            &struct_contract,
            ContractValidationInput::Struct(
                StructAspectValue::new([
                    (
                        field("title"),
                        AspectValue::String(InternedString::Raw("shared title payload".into())),
                    ),
                    (
                        field("notes"),
                        AspectValue::String(InternedString::Stored(StoredValueText::new(
                            "shared stored notes".into(),
                        ))),
                    ),
                ])
                .unwrap(),
            ),
        ) else {
            panic!("admitted struct")
        };
        let TransitionOutcome::Success(scalar_val) = validate_aspect_value(
            &scalar_contract,
            ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(
                "shared scalar".into(),
            ))),
        ) else {
            panic!("admitted scalar")
        };
        let TransitionOutcome::Success(admitted) =
            admit_authoritative_record_aspect_state([struct_val, scalar_val])
        else {
            panic!("admitted state")
        };
        let (state, _, _) = admitted.into_parts().into_parts();
        let exact_value_backing = state
            .aspects()
            .retained_storage_bytes()
            .unwrap()
            .checked_sub(
                struct_contract.owned_allocation_capacity_bytes()
                    + scalar_contract.owned_allocation_capacity_bytes(),
            )
            .unwrap() as u64;
        crate::storage::substrate::visit_foundational_value_backing(
            &state,
            false,
            &mut contracts_only,
        );
        assert_eq!(
            contracts_only.0.values().sum::<u64>(),
            exact_value_backing,
            "exact physical rows for the shared aspect map match its backing minus contracts"
        );
        (state, exact_value_backing)
    };
    assert!(shared_value_backing_bytes > 0);

    // Two distinct carriers hold clones of one persistent aspect map.
    let original = EntityExtra::from_unbudgeted_parts(None, None, Some(state.clone()));
    let copy = EntityExtra::from_unbudgeted_parts(None, None, Some(state.clone()));
    let key = AspectKey::new("label").unwrap();
    let held = |extra: &EntityExtra| {
        let state = extra.authoritative_aspect_state().unwrap();
        state.get(&key).unwrap().key().as_str().as_ptr()
    };
    assert_eq!(
        held(&original),
        held(&copy),
        "the carriers share the admitted aspect backing instead of duplicating it"
    );
    let layout = StorageAllocation::<EntityExtra>::layout_bytes();

    let mut current = arena(original);
    let prior = current.clone();
    let first = current.retained_extra(0).unwrap().id();
    current.apply_extra_update(0, copy, VersionId(2)).unwrap();
    let second = current.retained_extra(0).unwrap().id();
    assert_ne!(first, second, "two distinct carriers hold the shared map");

    // Incremental delta accounting matches exact full inventory subtraction.
    check_delta(&current, &prior);

    let mut observed = UniqueRows::default();
    current.visit_authoritative_allocations(false, &mut observed);
    let mut shared_ids = UniqueRows::default();
    crate::storage::substrate::visit_foundational_value_backing(&state, false, &mut shared_ids);

    // Both carriers appear as distinct rows with ONLY their own carrier layout;
    // the shared persistent aspect nodes and admitted values appear once.
    assert_eq!(observed.0.get(&first), Some(&layout));
    assert_eq!(observed.0.get(&second), Some(&layout));
    for (id, bytes) in &shared_ids.0 {
        assert_eq!(
            observed.0.get(id),
            Some(bytes),
            "every shared aspect node/value is present once in the arena union"
        );
    }
    let carrier_and_shared_union: u64 = observed.0[&first]
        + observed.0[&second]
        + shared_ids.0.keys().map(|id| observed.0[id]).sum::<u64>();
    assert_eq!(
        carrier_and_shared_union,
        2 * layout + shared_value_backing_bytes,
        "two Extras sharing one aspect map contribute the shared backing once"
    );
    assert!(
        carrier_and_shared_union < 2 * layout + 2 * shared_value_backing_bytes,
        "shared aspect backing must never be counted twice across sharing Extras"
    );

    // Confirm the inventory refuses an unrepresentable total with typed overflow.
    let half_max = crate::storage::substrate::RecordArenaAllocationInventory {
        authoritative_bytes: u64::MAX,
        ..Default::default()
    };
    let one = crate::storage::substrate::RecordArenaAllocationInventory {
        authoritative_bytes: 1,
        ..Default::default()
    };
    assert_eq!(half_max.checked_add(one), Err(StorageAllocationOverflow));
}
