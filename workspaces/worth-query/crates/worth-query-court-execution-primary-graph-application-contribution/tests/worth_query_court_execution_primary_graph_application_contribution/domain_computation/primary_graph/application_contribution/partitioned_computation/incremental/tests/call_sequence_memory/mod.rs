//! Actual measured call custody funds each persistent view and exact reach index.
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::test_placement::test_policy;
#[test]
fn measured_call_shape_view_keeps_its_own_grant_after_the_sequence_drops() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let sealed = first.sealed.unwrap().unwrap();
    let prior = (*(*sealed
        .state_for_test())
        .typed_for_test())
        .downcast_ref::<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 30),
        )),
        &request,
    );
    let sequence=worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::ComputationCallSequence::build_for_test(||std::iter::once((ComputationRead::Membership,&(*(*prior.membership_for_test()).charge_for_test()),(*(*prior.membership_for_test()).declared_units_for_test())))
        .chain((*prior.item_keys_for_test()).iter().map(|(item,call)|(ComputationRead::ItemKey(*item),&(*call.charge_for_test()),(*call.declared_units_for_test()))))
        .chain((*prior.partitions_for_test()).iter_for_test().map(|(id,partition)|(ComputationRead::Partition(*id),&(*partition.gather_for_test()),Some(0)))),&execution).unwrap();
    let shape = Arc::clone(sequence.shape_for_test());
    drop(sequence);
    assert!(
        execution.reserve(1 << 30).is_err(),
        "the remaining shape view still owns its backing grant"
    );
    assert!(shape.calls().next().is_some());
    let clone = Arc::clone(&shape);
    drop(shape);
    assert!(execution.reserve(1 << 30).is_err());
    drop(clone);
    assert!(execution.reserve(1 << 30).is_ok());
}
#[test]
fn measured_call_sequence_refuses_before_copy_and_holds_its_view_until_final_drop() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let sealed = first.sealed.unwrap().unwrap();
    let prior = (*(*sealed
        .state_for_test())
        .typed_for_test())
        .downcast_ref::<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let sequence = &(*prior.call_sequence_for_test());
    let before = sequence.bytes_for_test();
    let (identity, partition) = (*prior.partitions_for_test()).iter_for_test().next().unwrap();
    let change = || {
        std::iter::once((
            ComputationRead::Partition(*identity),
            &(*partition.gather_for_test()),
            Some(0),
        ))
    };
    let request = live_scope();
    let narrow = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, Some(1))),
        &request,
    );
    assert!(matches!(
        sequence.patched_for_test(change(), &narrow),
        Err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::CallSequenceDenial::Storage(worth_execution::ExecutionOrderedMapDenial::Allocation(_)))
    ));
    assert_eq!(sequence.bytes_for_test(), before);
    let fresh = QueryRequestExecution::open_for_test(
        RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 30),
        )),
        &request,
    );
    let view = sequence.patched_for_test(change(), &fresh).unwrap();
    assert!(Arc::ptr_eq(sequence.shape_for_test(), view.shape_for_test()));
    let clone = Arc::clone(&view);
    assert!(fresh.reserve(1 << 30).is_err());
    drop(view);
    assert!(fresh.reserve(1 << 30).is_err());
    assert_eq!(clone.bytes_for_test(), before);
    drop(clone);
    assert!(fresh.reserve(1 << 30).is_ok());
    assert_eq!(sequence.bytes_for_test(), before);
}
#[test]
fn measured_call_scope_index_requires_a_reaching_call_inside_the_carried_range() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let sealed = first.sealed.unwrap().unwrap();
    let prior = (*(*sealed
        .state_for_test())
        .typed_for_test())
        .downcast_ref::<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::retained::RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let entity = (*(*sealed
        .state_for_test())
        .facts_for_test())
        .facts_for_test()
        .find_map(|(_, fact, _)| match fact {
            WorthQueryApplicationObservedFact::Field { entity_id, .. } => Some(*entity_id),
            _ => None,
        })
        .unwrap();
    let sequence = &(*prior.call_sequence_for_test());
    assert!(sequence.contains_in(entity, &(sequence.prefix_for_test()..sequence.shape_for_test().len_for_test())));
    assert!(
        !sequence.contains_in(entity, &(0..sequence.prefix_for_test())),
        "naming the same old sequence grants no reach without the reaching call"
    );
    assert!(!sequence.contains_in(entity, &(sequence.prefix_for_test()..sequence.prefix_for_test())));
}
