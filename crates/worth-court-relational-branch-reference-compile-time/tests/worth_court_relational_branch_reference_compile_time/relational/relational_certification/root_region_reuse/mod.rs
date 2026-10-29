#[allow(unused_imports)]
pub(crate) use {worth_relational::facade::transactions::AspectFieldPatch, worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_relational::facade::transactions::EntityMutationIntent, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::inspection::RelationalMvccCostScope, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_main_batch, crate::relational_certification::ordinary_context::root_sharing_observation::inspect_main_regions, crate::relational_certification::ordinary_context::root_sharing_observation::inspect_main_sharing, worth_relational::facade::transactions::planned_single_field_locator};

#[test]
fn region_reuse_is_sensitive_to_empty_and_touched_deltas() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    let baseline = inspect_main_regions(&world.runtime);
    let baseline_sharing = inspect_main_sharing(&world.runtime);
    let noop_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![world.runtime.main_branch_identity()],
    );
    assert!(!baseline.is_empty());
    commit_main_batch(
        execution,
        &world.runtime,
        worth_relational::facade::transactions::WorkerIntentBatch::new("phase5-no-storage-delta"),
    );
    let after_noop = inspect_main_regions(&world.runtime);
    let after_noop_sharing = inspect_main_sharing(&world.runtime);
    let noop_cost = world.runtime.observe_mvcc_cost(&noop_scope).unwrap();
    assert_eq!(after_noop, baseline);
    assert_eq!(
        noop_cost
            .sharing_cost_delta()
            .publication_touched_region_count,
        0
    );
    assert_eq!(
        noop_cost
            .sharing_cost_delta()
            .publication_reused_region_count,
        baseline.len() as u64
    );
    assert_eq!(
        after_noop_sharing.publication_touched_region_count(),
        baseline_sharing.publication_touched_region_count()
    );
    assert!(
        after_noop_sharing.publication_reused_region_count()
            > baseline_sharing.publication_reused_region_count()
    );

    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("name").unwrap(),
            worth_foundational::facade::FieldKey::new("name").unwrap(),
        ),
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw("mutation-denies-reuse".to_owned()),
        ),
    );
    let port_id = world.handles.entities[&worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey::new(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port, 0)].id;
    let mutation_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![world.runtime.main_branch_identity()],
    );
    commit_main_batch(
        execution,
        &world.runtime,
        worth_relational::facade::transactions::WorkerIntentBatch::new(
            "phase5-touched-storage-delta",
        )
        .push(
            worth_relational::facade::transactions::MutationIntent::Entity(
                worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: port_id,
                        fields: worth_relational::facade::transactions::AspectFieldPatch::new(
                            fields,
                        ),
                    },
                ),
            ),
        ),
    );
    let mutation = inspect_main_sharing(&world.runtime);
    let cost = world.runtime.observe_mvcc_cost(&mutation_scope).unwrap();
    assert_eq!(
        cost.sharing_cost_delta().publication_touched_region_count,
        1
    );
    assert_eq!(
        cost.sharing_cost_delta().publication_reused_region_count,
        baseline.len().saturating_sub(1) as u64
    );
    let after_mutation = mutation.region_locators().to_vec();
    assert_ne!(after_mutation, after_noop);
    let untouched =
        |locators: &[worth_relational::facade::inspection::RelationalStorageRegionLocator]| {
            locators
                .iter()
                .copied()
                .filter(|locator| locator.partition_id().as_u32() != 0)
                .collect::<std::collections::BTreeSet<_>>()
        };
    assert_eq!(untouched(&after_mutation), untouched(&baseline));
}
