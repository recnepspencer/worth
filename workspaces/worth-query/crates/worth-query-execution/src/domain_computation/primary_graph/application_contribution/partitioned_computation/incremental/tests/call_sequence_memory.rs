//! Actual measured call custody funds each persistent view and exact reach index.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy;
#[test]
fn measured_call_shape_view_keeps_its_own_grant_after_the_sequence_drops() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let sealed = first.sealed.unwrap().unwrap();
    let prior = sealed
        .state
        .typed
        .downcast_ref::<super::super::retained::RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 30),
        )),
        &request,
    );
    let sequence=crate::domain_computation::primary_graph::invariant_projection::ComputationCallSequence::build(||std::iter::once((ComputationRead::Membership,&prior.membership.charge,prior.membership.declared_units))
        .chain(prior.item_keys.iter().map(|(item,call)|(ComputationRead::ItemKey(*item),&call.charge,call.declared_units)))
        .chain(prior.partitions.iter().map(|(id,partition)|(ComputationRead::Partition(*id),&partition.gather,Some(0)))),&execution).unwrap();
    let shape = Arc::clone(sequence.shape());
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
    let prior = sealed
        .state
        .typed
        .downcast_ref::<super::super::retained::RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let sequence = &prior.call_sequence;
    let before = sequence.bytes();
    let (identity, partition) = prior.partitions.iter().next().unwrap();
    let change = || {
        std::iter::once((
            ComputationRead::Partition(*identity),
            &partition.gather,
            Some(0),
        ))
    };
    let request = live_scope();
    let narrow = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(std::num::NonZeroUsize::MIN, Some(1))),
        &request,
    );
    assert!(matches!(
        sequence.patched(change(), &narrow),
        Err(crate::domain_computation::primary_graph::invariant_projection::CallSequenceDenial::Storage(worth_execution::ExecutionOrderedMapDenial::Allocation(_)))
    ));
    assert_eq!(sequence.bytes(), before);
    let fresh = QueryRequestExecution::open(
        RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 30),
        )),
        &request,
    );
    let view = sequence.patched(change(), &fresh).unwrap();
    assert!(Arc::ptr_eq(sequence.shape(), view.shape()));
    let clone = Arc::clone(&view);
    assert!(fresh.reserve(1 << 30).is_err());
    drop(view);
    assert!(fresh.reserve(1 << 30).is_err());
    assert_eq!(clone.bytes(), before);
    drop(clone);
    assert!(fresh.reserve(1 << 30).is_ok());
    assert_eq!(sequence.bytes(), before);
}
#[test]
fn measured_call_scope_index_requires_a_reaching_call_inside_the_carried_range() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let first = first_run(&world, &installed(StatusRead::Gather(1), sum));
    let sealed = first.sealed.unwrap().unwrap();
    let prior = sealed
        .state
        .typed
        .downcast_ref::<super::super::retained::RetainedPartitions<Parity, Number, u64>>()
        .unwrap();
    let entity = sealed
        .state
        .facts
        .facts()
        .find_map(|(_, fact, _)| match fact {
            WorthQueryApplicationObservedFact::Field { entity_id, .. } => Some(*entity_id),
            _ => None,
        })
        .unwrap();
    let sequence = &prior.call_sequence;
    assert!(sequence.contains_in(entity, &(sequence.prefix()..sequence.shape().len())));
    assert!(
        !sequence.contains_in(entity, &(0..sequence.prefix())),
        "naming the same old sequence grants no reach without the reaching call"
    );
    assert!(!sequence.contains_in(entity, &(sequence.prefix()..sequence.prefix())));
}
