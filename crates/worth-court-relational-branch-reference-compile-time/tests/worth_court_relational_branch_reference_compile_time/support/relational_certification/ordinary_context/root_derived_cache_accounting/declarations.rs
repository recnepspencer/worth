pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::FieldKey};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId};
pub(crate) use {worth_relational::facade::indexes::DerivedIndexBuildRequest, worth_relational::facade::indexes::DerivedIndexDefinition, worth_relational::facade::indexes::DerivedIndexEntries, worth_relational::facade::indexes::DerivedIndexId, worth_relational::facade::indexes::DerivedIndexKind};
pub(crate) use {worth_relational::facade::inspection::RelationalExcludedAllocationLane, worth_relational::facade::inspection::RelationalOwnerAllocationLedgerObservation};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator};

pub(crate) fn empty_entity_field_cache_growth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> u64 {
    let (world, expected) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
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
            kind: worth_relational::facade::indexes::DerivedIndexKind::EntityField {
                field_locator: planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("phase5-absent")
                        .expect("valid absent aspect key"),
                    worth_foundational::facade::FieldKey::new("phase5-absent")
                        .expect("valid absent field key"),
                ),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    let build = world.runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: source_commit.commit_id,
            branch_id: main_branch,
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    assert!(matches!(
        &build.generations[0].entries,
        worth_relational::facade::indexes::DerivedIndexEntries::EntityField(entries) if entries.is_empty()
    ));
    world
        .runtime
        .observe_branch_sharing(std::slice::from_ref(&main_identity))
        .expect("empty derived generation is inspectable")
        .unique_optional_cache_bytes()
        .saturating_sub(before)
}

pub(crate) fn entity_field_nested_initialized_byte_floor(
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

pub(crate) fn optional_cache_bytes(ledger: &RelationalOwnerAllocationLedgerObservation) -> u64 {
    ledger
        .excluded_allocations()
        .iter()
        .filter(|allocation| allocation.lane() == worth_relational::facade::inspection::RelationalExcludedAllocationLane::OptionalCache)
        .map(|allocation| allocation.bytes())
        .sum()
}
