use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::TestsWorthQueryObservedSourceSelectionForTest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_reuse_key::PreparedInputReuseKey;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;

#[test]
fn equal_source_membership_cannot_reuse_a_different_input_digest() {
    let selection = WorthQueryObservedSourceSelection::selecting_for_test(1);
    let edition = InstalledProducerEdition::for_test([0x51; 32]);
    let original = PreparedInputReuseKey::new(selection.clone(), [0x61; 32], edition);
    let continued = original.continued_by_republication();
    assert!(
        original
            .same_prepared_input_as(&continued, &mut |_| Ok(()))
            .unwrap(),
        "equal complete prepared input keys must permit reuse"
    );
    let changed = PreparedInputReuseKey::new(selection, [0x62; 32], edition);
    assert!(
        !original
            .same_prepared_input_as(&changed, &mut |_| Ok(()))
            .unwrap(),
        "reuse must compare the input digest even when source membership is unchanged"
    );
}
