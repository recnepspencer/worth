#[allow(unused_imports)]
pub(crate) use {worth_foundational::facade::AspectKey, worth_relational::facade::history::BranchId, worth_relational::facade::indexes::DerivedIndexBuildRequest, worth_relational::facade::indexes::DerivedIndexDefinition, worth_relational::facade::indexes::DerivedIndexEntries, worth_relational::facade::indexes::DerivedIndexId, worth_relational::facade::indexes::DerivedIndexKind, worth_foundational::facade::FieldKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, crate::relational_certification::ordinary_context::root_derived_cache_accounting::empty_entity_field_cache_growth, crate::relational_certification::ordinary_context::root_derived_cache_accounting::entity_field_nested_initialized_byte_floor, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, crate::relational_certification::ordinary_context::root_derived_cache_accounting::optional_cache_bytes, worth_relational::facade::transactions::planned_single_field_locator};

#[test]
fn populated_derived_index_cache_stays_outside_owner_authority_accounting() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let empty_cache_growth = empty_entity_field_cache_growth(execution);
    let (world, expected) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
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
            kind: worth_relational::facade::indexes::DerivedIndexKind::EntityField {
                field_locator: planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("status")
                        .expect("Supply Chain status aspect"),
                    worth_foundational::facade::FieldKey::new("status")
                        .expect("Supply Chain status field"),
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
    assert_eq!(build.generations.len(), 1);
    assert!(matches!(
        &build.generations[0].entries,
        worth_relational::facade::indexes::DerivedIndexEntries::EntityField(entries) if !entries.is_empty()
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
        worth_relational::facade::indexes::DerivedIndexEntries::EntityField(entries) => {
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
