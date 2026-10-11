use super::root_sharing_observation::{inspect_main_regions, inspect_main_sharing};
use super::world::supply_chain::commit_main_batch;
use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
use super::world::supply_chain::{EntityKey, EntityKind, SupplyChainScale};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_relational::facade::inspection::RelationalMvccCostScope;
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, MutationIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

#[test]
fn phase5_region_reuse_is_sensitive_to_empty_and_touched_deltas() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, expected) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &expected);
    let baseline = inspect_main_regions(&world.runtime);
    let baseline_sharing = inspect_main_sharing(&world.runtime);
    let noop_scope = RelationalMvccCostScope::capture(
        &world.runtime,
        vec![world.runtime.main_branch_identity()],
    );
    assert!(!baseline.is_empty());
    commit_main_batch(
        execution,
        &world.runtime,
        WorkerIntentBatch::new("phase5-no-storage-delta"),
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
            AspectKey::new("name").unwrap(),
            FieldKey::new("name").unwrap(),
        ),
        AspectValue::String(InternedString::Raw("mutation-denies-reuse".to_owned())),
    );
    let port_id = world.handles.entities[&EntityKey::new(EntityKind::Port, 0)].id;
    let mutation_scope = RelationalMvccCostScope::capture(
        &world.runtime,
        vec![world.runtime.main_branch_identity()],
    );
    commit_main_batch(
        execution,
        &world.runtime,
        WorkerIntentBatch::new("phase5-touched-storage-delta").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: port_id,
                fields: AspectFieldPatch::new(fields),
            }),
        )),
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
