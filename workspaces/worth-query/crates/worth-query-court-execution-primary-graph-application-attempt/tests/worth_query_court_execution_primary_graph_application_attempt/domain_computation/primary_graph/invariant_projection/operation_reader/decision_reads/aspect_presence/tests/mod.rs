use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::DecisionReadOutcome;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::DecisionReads;
    use worth_foundational::facade::AspectKey;
    use worth_relational::facade::identity::EntityId;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationFactKey as Key;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;
    use std::collections::BTreeSet;
    use worth_foundational::facade::FieldKey;
    use worth_relational::facade::identity::PartitionId;
    use worth_relational::facade::transactions::planned_single_field_locator;

pub fn presence(slot: u64, aspect: &str, field: &str) -> Key {
    Key::AspectPresence {
        entity: "Draft".to_owned(),
        entity_id: EntityId::new(PartitionId::main(), slot, 1),
        locator: planned_single_field_locator(
            AspectKey::new(aspect).unwrap(),
            FieldKey::new(field).unwrap(),
        ),
    }
}

#[test]
fn optional_aspect_initialization_coalesces_presence_without_dropping_attribution() {
    let mut reads = DecisionReads::default();
    let first = presence(1, "facts", "note");
    let second = presence(1, "facts", "score");
    reads.record(first.clone(), DecisionReadOutcome::Observed);
    (*reads.reading_mut_for_test()) = Some(ComputationRead::Membership);
    reads.record(second.clone(), DecisionReadOutcome::Observed);
    (*reads.reading_mut_for_test()) = None;
    assert!((*reads.computation_for_test()).contains_key(&first));
    assert!(!(*reads.computation_for_test()).contains_key(&second));
    let other_aspect = presence(1, "other", "note");
    let other_entity = presence(2, "facts", "note");
    reads.record(other_aspect.clone(), DecisionReadOutcome::Observed);
    reads.record(other_entity.clone(), DecisionReadOutcome::Observed);
    let (keys, _) = reads
            .into_expected_for_test(&worth_execution::ExecutionRequest::serial(
                &worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request(),
            ))
            .ok()
            .unwrap();
    assert_eq!(
        keys.iter_for_test().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([first, other_aspect, other_entity])
    );
}
