//! Refusing discovery cannot classify a displaced settlement as unposted.
use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::{
    WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
};
use crate::domain_computation::primary_graph::output_lineage::registry_fixture;
use crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner;

#[test]
fn registry_post_commit_discovery_keeps_one_holder_for_an_unposted_displaced() {
    let (_lineage, displaced) = registry_fixture::recorded_settlement();
    let registry = WorthQueryOutputDemandRegistry::default();
    let selected = Arc::new(key("retirement-result", 2, 1));
    let mut prelude = record_admission();
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(
            selected.as_ref().clone(),
            record(occurrence(), DemandState::Running, 1),
        );
        state.charge_record_lookup(&selected, &mut prelude).unwrap();
    }
    let resources = WorthQueryInvalidationResources::install(
        WorthQueryInvalidationResourceInstallation::bounded(
            prelude.charged_work() + 1,
            1,
            1 << 20,
            1,
        ),
    )
    .unwrap();
    let owner = SourceInvalidationOwner::new(resources, 1);
    let totals = || {
        let state = registry.state.lock().unwrap();
        (
            state.required_reserved_bytes,
            state.obligation_reserved_bytes,
            state
                .required_custody_retained_bytes
                .load(std::sync::atomic::Ordering::Acquire),
        )
    };
    let baseline = totals();
    let pins = Arc::strong_count(&displaced);
    // This invokes the same completion entry as both publication callers. The real
    // recorded identity has no demand posting; the owner must retain its
    // release when this tiny later-maintenance admission cannot retire it.
    super::super::SupersededSettlements::new(registry.clone(), selected)
        .retire(Some(Arc::clone(&displaced)), &owner);
    assert_eq!(
        Arc::strong_count(&displaced),
        pins + 1,
        "a committed displacement must have exactly one retry holder"
    );
    assert_eq!(totals(), baseline);
    let retired = owner.retire_released(std::iter::empty(), &mut record_admission());
    assert_eq!(retired, [Arc::clone(&displaced)]);
    drop(retired);
    assert_eq!(Arc::strong_count(&displaced), pins);
    assert!(owner
        .retire_released(std::iter::empty(), &mut record_admission())
        .is_empty());
    assert_eq!(totals(), baseline);
}
