//! Refusing discovery cannot classify a displaced settlement as unposted.
use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::{
    WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
};
use crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner;

#[test]
fn refused_retirement_never_releases_the_displaced_settlement() {
    let (_lineage, displaced) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
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
    // The keyed prelude fits; the two occurrence seeks do not. Tiny scratch
    // also prevents a mistakenly released identity from immediately retiring,
    // making its real custody in the owner's release queue observable.
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
    assert!(!registry
        .posts_settlement(&displaced, &mut record_admission())
        .unwrap());
    let retained = Arc::strong_count(&displaced);
    let denial = super::super::SupersededSettlements::new(registry.clone(), selected)
        .retire(Some(Arc::clone(&displaced)), &owner)
        .expect_err("discovery must return its typed work refusal");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(
        Arc::strong_count(&displaced),
        retained,
        "a refused discovery must not put displaced into the released queue"
    );
    assert!(!registry
        .posts_settlement(&displaced, &mut record_admission())
        .unwrap());
}
