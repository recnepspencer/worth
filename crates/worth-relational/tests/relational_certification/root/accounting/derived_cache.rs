use super::world::supply_chain::SupplyChainScale;
use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, head_for_supply_chain_branch,
};
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexEntries, DerivedIndexId,
    DerivedIndexKind,
};
use worth_relational::facade::inspection::{
    RelationalExcludedAllocationLane, RelationalOwnerAllocationLedgerObservation,
};
use worth_relational::facade::transactions::planned_single_field_locator;

#[test]
fn populated_derived_index_cache_stays_outside_owner_authority_accounting() {
    let empty_cache_growth = empty_entity_field_cache_growth();
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let main_branch = BranchId("main".to_owned());
    let main_identity = world.runtime.main_branch_identity();
    let source_commit = head_for_supply_chain_branch(&world.runtime, &main_branch);
    let before_ledger = world
        .runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(&main_identity))
        .expect("owner ledger observes the selected root directly");
    let before_sharing = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&main_identity))
        .expect("sharing boundary observes the selected root");

    let index = world
        .runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "supply-chain.status.optional-cache".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: planned_single_field_locator(
                    AspectKey::new("status").expect("Supply Chain status aspect"),
                    FieldKey::new("status").expect("Supply Chain status field"),
                ),
            },
            branch_scoped: true,
        });
    let build = world
        .runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: source_commit.commit_id,
            branch_id: main_branch,
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());
    assert_eq!(build.generations.len(), 1);
    assert!(matches!(
        &build.generations[0].entries,
        DerivedIndexEntries::EntityField(entries) if !entries.is_empty()
    ));

    let after_ledger = world
        .runtime
        .inspect_owner_allocation_ledger(std::slice::from_ref(&main_identity))
        .expect("owner ledger observes the populated optional cache");
    let after_sharing = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&main_identity))
        .expect("sharing boundary observes the populated optional cache");

    assert_eq!(
        before_ledger.canonical_payloads(),
        after_ledger.canonical_payloads()
    );
    assert_eq!(
        before_ledger.authoritative_allocations(),
        after_ledger.authoritative_allocations(),
        "every owner-issued authoritative locator and byte count remains exact"
    );
    assert_eq!(
        before_sharing.authoritative_allocations(),
        after_sharing.authoritative_allocations(),
        "the derived sharing boundary cannot promote cache bytes into authority"
    );
    assert_eq!(before_sharing.root_ids(), after_sharing.root_ids());
    assert_eq!(
        before_sharing.unique_physical_authoritative_bytes(),
        after_sharing.unique_physical_authoritative_bytes()
    );
    assert_eq!(
        before_sharing.branch_metadata_bytes(),
        after_sharing.branch_metadata_bytes()
    );
    assert!(
        after_sharing.unique_optional_cache_bytes() > before_sharing.unique_optional_cache_bytes()
    );
    let cache_growth = after_sharing
        .unique_optional_cache_bytes()
        .saturating_sub(before_sharing.unique_optional_cache_bytes());
    let nested_entry_bytes = match &build.generations[0].entries {
        DerivedIndexEntries::EntityField(entries) => {
            entity_field_nested_initialized_byte_floor(entries)
        }
        _ => panic!("the registered entity-field index produced another entry family"),
    };
    assert!(
        nested_entry_bytes > 0,
        "the adversarial floor is entry-sensitive"
    );
    assert!(
        cache_growth >= empty_cache_growth.saturating_add(nested_entry_bytes),
        "a populated generation must exceed the structurally identical empty generation by its entry, initialized key, and initialized value storage"
    );
    assert_eq!(
        after_sharing.unique_optional_cache_bytes(),
        optional_cache_bytes(&after_ledger),
        "the sharing summary agrees with the independently assembled owner ledger"
    );
}

fn empty_entity_field_cache_growth() -> u64 {
    let (world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let main_branch = BranchId("main".to_owned());
    let main_identity = world.runtime.main_branch_identity();
    let source_commit = head_for_supply_chain_branch(&world.runtime, &main_branch);
    let before = world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&main_identity))
        .expect("empty-cache baseline is inspectable")
        .unique_optional_cache_bytes();
    let index = world
        .runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "supply-chain.absent.optional-cache-control".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: planned_single_field_locator(
                    AspectKey::new("phase5-absent").expect("valid absent aspect key"),
                    FieldKey::new("phase5-absent").expect("valid absent field key"),
                ),
            },
            branch_scoped: true,
        });
    let build = world
        .runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: source_commit.commit_id,
            branch_id: main_branch,
            index_ids: vec![index.index_id],
        });
    assert!(build.failed_indexes.is_empty());
    assert!(matches!(
        &build.generations[0].entries,
        DerivedIndexEntries::EntityField(entries) if entries.is_empty()
    ));
    world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&main_identity))
        .expect("empty derived generation is inspectable")
        .unique_optional_cache_bytes()
        .saturating_sub(before)
}

fn entity_field_nested_initialized_byte_floor(
    entries: &worth_relational::facade::indexes::DerivedIndexEntryMap<
        worth_relational::facade::storage::AuthoritativeFieldComparisonKey,
        EntityId,
    >,
) -> u64 {
    let map_entries = (entries.len() as u64).saturating_mul(std::mem::size_of::<(
        worth_relational::facade::storage::AuthoritativeFieldComparisonKey,
        worth_relational::facade::indexes::DerivedIndexRows<EntityId>,
    )>() as u64);
    entries.iter().fold(map_entries, |bytes, (key, ids)| {
        bytes
            .saturating_add(key.canonical_value_bytes().len() as u64)
            .saturating_add(
                (ids.len() as u64).saturating_mul(std::mem::size_of::<EntityId>() as u64),
            )
    })
}

fn optional_cache_bytes(ledger: &RelationalOwnerAllocationLedgerObservation) -> u64 {
    ledger
        .excluded_allocations()
        .iter()
        .filter(|allocation| allocation.lane() == RelationalExcludedAllocationLane::OptionalCache)
        .map(|allocation| allocation.bytes())
        .sum()
}
