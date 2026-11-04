pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::custody_wait;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::demand_key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody;

pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
pub use std::cmp::Ordering;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::root_revision_order;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
#[test]
fn newer_root_retires_only_its_matching_source() {
    let (scope, occurrence) = real_scope_and_occurrence();
    let older = BoundOutputSource::new_for_test(scope, key("producer", 5, 1).source_into_for_test());
    let sibling = BoundOutputSource::new_for_test(scope, key("producer", 5, 2).source_into_for_test());
    let newer = BoundOutputSource::new_for_test(scope, key("producer", 6, 1).source_into_for_test());
    assert_eq!(root_revision_order(&older, &newer), Some(Ordering::Less));
    assert_eq!(
        root_revision_order(&sibling, &sibling),
        Some(Ordering::Equal)
    );
    assert_eq!(root_revision_order(&older, &sibling), None);
    let mut custody = SourceCustody::new_for_test(occurrence, PreparedOutputRootKind::Discovered(TypeId::of_for_test::<()>()), None, None, Some(vec![older.clone(), sibling.clone()]), Vec::new(), Vec::new(), None, 0, Default::default(), false);
    custody.retire_superseded_roots_for_test(&[newer, sibling.clone()]);
    assert_eq!(
        custody.source_denial_for_test(&(*older.identity_for_test())).unwrap().kind(),
        WorthQueryOutputDemandDenialKind::Superseded
    );
    assert!(custody.source_denial_for_test(&(*sibling.identity_for_test())).is_none());
    assert!(
        (*custody.retired_for_test()).is_none(),
        "one stale root cannot retire its sibling custody"
    );
}
#[test]
fn registry_custody_waits_for_its_token_without_an_active_record() {
    for superseded in [false, true] {
        custody_wait(superseded, true);
    }
}
#[test]
fn registry_custody_waits_for_its_active_record_without_a_token() {
    for superseded in [false, true] {
        custody_wait(superseded, false);
    }
}

#[path = "retained_proofs/mod.rs"]
pub mod retained_proofs;
