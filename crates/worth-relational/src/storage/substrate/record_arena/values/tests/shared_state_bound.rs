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

const ALLOCATOR_UNION_PROBE_GATE: &str = "WORTH_RELATIONAL_SHARED_STATE_ALLOCATOR_UNION_PROBE";

#[test]
fn shared_aspect_map_union_matches_independent_os_allocator_measurement() {
    let output = crate::process_deadline::output(
        std::process::Command::new(
            std::env::current_exe().expect("the test binary knows its own path"),
        )
        .arg("isolated_shared_aspect_map_os_allocator_union_probe")
        .arg("--test-threads=1")
        .env(ALLOCATOR_UNION_PROBE_GATE, "1"),
    )
    .expect("the isolated allocator-union probe process should start");
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "isolated allocator-union probe failed:\n{report}\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        report.contains("1 passed"),
        "isolated allocator-union probe ran no test:\n{report}",
    );
}

#[test]
fn isolated_shared_aspect_map_os_allocator_union_probe() {
    use stats_alloc::{Region, INSTRUMENTED_SYSTEM};
    if std::env::var_os(ALLOCATOR_UNION_PROBE_GATE).is_none() {
        return;
    }
    let field = |name: &str| FieldKey::new(name).unwrap();
    // Construct schema contracts and lookup keys BEFORE starting the OS allocator
    // measurement region so only the shared value backing + two Extra carriers are measured.
    let struct_contract = AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(17),
        AspectContractRevision(1),
        StructAspectShape::new(["title", "notes", "ratio"].map(|name| {
            let scalar_type = if name == "ratio" {
                ScalarAspectType::Rational
            } else {
                ScalarAspectType::String
            };
            FieldDeclaration::new(
                field(name),
                scalar_type,
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
        AspectIdentity(18),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );

    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let (carrier_a, carrier_b) = {
        let TransitionOutcome::Success(struct_val) = validate_aspect_value(
            &struct_contract,
            ContractValidationInput::Struct(
                StructAspectValue::new([
                    (
                        field("title"),
                        AspectValue::String(InternedString::Raw("independent os raw title".into())),
                    ),
                    (
                        field("notes"),
                        AspectValue::String(InternedString::Stored(StoredValueText::new(
                            "independent os stored notes".into(),
                        ))),
                    ),
                    (
                        field("ratio"),
                        AspectValue::Rational(
                            CanonicalRational::new(
                                CanonicalBigInt::new("123456789012345678901234567890"),
                                CanonicalBigInt::new("98765432109876543210987654321"),
                            )
                            .unwrap(),
                        ),
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
                "independent os scalar payload".into(),
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
        let a = StorageAllocation::new(EntityExtra::from_unbudgeted_parts(
            None,
            None,
            Some(state.clone()),
        ));
        let b = StorageAllocation::new(EntityExtra::from_unbudgeted_parts(
            None,
            None,
            Some(state),
        ));
        (a, b)
    };
    // Measure exact live OS-allocated bytes for the two distinct Extra carriers
    // plus their single shared nested persistent aspect map (excluding SchemaContracts).
    let os_stats = region.change();
    assert!(os_stats.bytes_allocated >= os_stats.bytes_deallocated);
    let os_live_bytes = (os_stats.bytes_allocated - os_stats.bytes_deallocated) as u64;

    // Independently walk each carrier asserting zero duplicate physical IDs within
    // one carrier walk, then union across both carriers by physical ID.
    let mut single_a = BTreeMap::new();
    let mut single_b = BTreeMap::new();
    struct StrictPerCarrier<'a>(&'a mut BTreeMap<u64, u64>);
    impl StorageAllocationVisitor for StrictPerCarrier<'_> {
        fn visit(&mut self, obs: StorageAllocationObservation) -> bool {
            assert_eq!(obs.extent, StorageAllocationExtent::Checked);
            assert!(
                self.0.insert(obs.id, obs.bytes).is_none(),
                "single-carrier walk must never emit duplicate physical allocation id {:#x}",
                obs.id
            );
            true
        }
    }
    let walk_carrier = |carrier: &StorageAllocation<EntityExtra>, out: &mut BTreeMap<u64, u64>| {
        let mut visitor = StrictPerCarrier(out);
        if visitor.visit(carrier.observation(false)) {
            EntityRecordKind::visit_extra_allocations(carrier, false, &mut visitor);
        }
    };
    walk_carrier(&carrier_a, &mut single_a);
    walk_carrier(&carrier_b, &mut single_b);
    assert_ne!(carrier_a.id(), carrier_b.id());

    let mut union = single_a.clone();
    for (id, bytes) in single_b {
        if let Some(prev) = union.insert(id, bytes) {
            assert_eq!(prev, bytes);
        }
    }
    let visited_union_bytes: u64 = union.values().sum();
    assert_eq!(
        visited_union_bytes, os_live_bytes,
        "exact physical allocation union across two sharing Extra carriers must equal OS allocator live bytes 1:1"
    );

    drop((single_a, union, carrier_a, carrier_b));
    let after_drop = region.change();
    assert_eq!(
        after_drop.bytes_allocated,
        after_drop.bytes_deallocated,
        "dropping both Extra carriers and scratch maps releases 100% of measured OS bytes"
    );
}
