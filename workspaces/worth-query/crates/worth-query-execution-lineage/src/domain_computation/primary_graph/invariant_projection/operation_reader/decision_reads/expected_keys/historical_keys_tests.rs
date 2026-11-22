//! Historical conversion retains exact handler keys through value aliasing.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn historical_conversion_and_aliases_preserve_exact_handler_keys() {
    let key = Key::Adjacency {
        relation: "historical-retention".into(),
        anchor: EntityId::new(PartitionId::main(), 1, 0),
        direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
        maximum_work_units: 2,
    };
    let handler = Arc::new(BTreeSet::from([key.clone()]));
    let mut attribution = ComputationFactAttribution::default();
    attribution.retain_handler(Arc::clone(&handler));
    let attribution = Arc::new(attribution);
    let expected = ExpectedFactKeys::new(
        handler,
        Arc::clone(&attribution),
        None,
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let historical = expected.into_historical_keys();
    assert!(Arc::ptr_eq(&historical, &attribution));
    assert_eq!(
        historical
            .historical_handler_keys()
            .cloned()
            .collect::<Vec<_>>()
            .as_slice(),
        std::slice::from_ref(&key)
    );
    let alias = Arc::clone(&historical);
    drop((historical, attribution));
    assert_eq!(
        alias.historical_handler_keys().cloned().collect::<Vec<_>>(),
        [key]
    );
}
