use crate::data::node::EvaluationCondition;
use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation, RetainedStoragePreparationDenial,
};

use super::NodeDefinitionData;

#[test]
fn retained_definition_charge_preserves_hidden_condition_capacity_and_bounds_preparation() {
    let mut condition = String::with_capacity(32_768);
    condition.push_str("installed-condition");
    let retained_capacity = condition.capacity() as u64;
    let mut definition = NodeDefinitionData::default();
    definition.eval_config.condition = EvaluationCondition::Custom(condition);
    let mut definitions: PersistentPagedVector<_> = [definition].into_iter().collect();
    let mut selected = definitions.fork_persistent();
    selected[0].eval_config.condition = EvaluationCondition::Always;
    drop(definitions);

    let mut work = RetainedStoragePreparation::new(1_000);
    let charge = selected.retained_heap_charge(&mut work).unwrap();
    assert!(charge.bytes() >= retained_capacity);
    let required_visits = work.visits();
    let mut insufficient = RetainedStoragePreparation::new(required_visits - 1);
    assert_eq!(
        selected.retained_heap_charge(&mut insufficient),
        Err(RetainedStoragePreparationDenial::WorkExhausted {
            maximum_visits: required_visits - 1,
        }),
    );
    assert_eq!(insufficient.visits(), required_visits - 1);
    let mut exact = RetainedStoragePreparation::new(required_visits);
    assert_eq!(selected.retained_heap_charge(&mut exact).unwrap(), charge);

    // Removing the selected node does not release the immutable base's
    // installed meaning. Clearing the storage is the release boundary.
    selected.pop_back().unwrap();
    assert!(selected.is_empty());
    assert!(
        selected
            .retained_heap_charge(&mut RetainedStoragePreparation::new(1_000))
            .unwrap()
            .bytes()
            >= retained_capacity
    );
    selected.clear();
    assert_eq!(
        selected
            .retained_heap_charge(&mut RetainedStoragePreparation::new(2))
            .unwrap()
            .bytes(),
        0,
    );
}
