//! Refusing discovery cannot classify a displaced settlement as unposted.
use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::WorthQueryInvalidationResources;
use crate::domain_computation::primary_graph::output_lineage::registry_fixture;
use crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner;

#[test]
fn registry_post_commit_discovery_releases_an_unposted_displaced() {
    let (_lineage, displaced) = registry_fixture::recorded_settlement();
    let registry = WorthQueryOutputDemandRegistry::default();
    let selected = Arc::new(key("retirement-result", 2, 1));

    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(
            selected.as_ref().clone(),
            record(occurrence(), DemandState::Running, 1),
        );
    }
    let resources = WorthQueryInvalidationResources::new();
    let owner = SourceInvalidationOwner::new(resources, 1);

    let pins = Arc::strong_count(&displaced);
    // This invokes the same completion entry as both publication callers. The real
    // recorded identity has no demand posting; the owner must retain its
    // release when this tiny later-maintenance admission cannot retire it.
    super::super::SupersededSettlements::new(registry.clone(), selected)
        .retire(Some(Arc::clone(&displaced)), &owner);
    assert_eq!(
        Arc::strong_count(&displaced),
        pins,
        "an unposted displaced settlement releases its original holder"
    );
    let retired = owner.retire_released(std::iter::empty());
    assert!(
        retired.is_empty(),
        "completed retirement leaves no synthetic retry holder"
    );
    drop(retired);
    assert_eq!(Arc::strong_count(&displaced), pins);
    assert!(owner.retire_released(std::iter::empty()).is_empty());
}
