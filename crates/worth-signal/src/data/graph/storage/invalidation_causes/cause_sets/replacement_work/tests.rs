use super::*;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::retained_storage::RetainedStoragePreparation as Work;
fn cause(node: u32, ordinal: u64) -> ResolvedDependencyCause {
    ResolvedDependencyCause::new(
        1,
        NodeId::new(node, 0),
        DependencyRevision(1),
        NodeId::new(9, 0),
        crate::data::aspect::Aspect::new(1),
        None,
        0,
        OutputCommitOrdinal(ordinal),
        1,
        Default::default(),
    )
}
fn normalized(causes: Vec<ResolvedDependencyCause>) -> NormalizedCauseSet {
    NormalizedCauseSet::prepare(causes, &mut EvaluationWork::Ordinary).unwrap()
}
#[test]
fn mixed_replacement_batch_admits_before_release_and_slot_reuse() {
    let mut source = CanonicalCauseSetStore::default();
    let producer = source.insert([cause(0, 1)]);
    let replace = source.insert([cause(1, 1)]);
    let release = source.insert([cause(2, 1)]);
    let scope = "published-scope".repeat(1000);
    let aspect = crate::data::aspect::Aspect::new(1);
    let delta = crate::data::proof::invalidation::output_commit::ProducedAspectDelta::from_committed_result(
        NodeId::new(9, 0), source.reserve_output_commit_ordinal(), crate::data::aspect::AspectVersion::zero(), crate::data::aspect::AspectVersion::from_updates([(aspect, 1)]), crate::data::aspect::AspectMask::from_aspect(aspect), &[(aspect, crate::data::output::ChangedRegion::new(scope.as_str()))], &[],
    ).unwrap();
    source.publish_output_commit(delta.clone());
    let mut graph_store = source.fork_persistent();
    let incoming = [
        normalized(vec![cause(1, 2)]),
        normalized(vec![]),
        normalized(vec![cause(3, 2)]),
    ];
    let selected = [
        (replace, &incoming[0]),
        (release, &incoming[1]),
        (PendingCauseSetId::EMPTY, &incoming[2]),
    ];
    let mut measured = Work::new(usize::MAX);
    graph_store
        .admit_replacement_batch_work(
            producer,
            &selected,
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
    let cost = measured.visits();
    assert!(cost >= scope.len());
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 17);
        work.reserve_visits(cost + 17 - available).unwrap();
        let result = graph_store.admit_replacement_batch_work(
            producer,
            &selected,
            &mut EvaluationWork::Conditional(&mut work),
        );
        if available == cost {
            result.unwrap();
            assert_eq!(work.visits(), cost + 17);
        } else {
            assert!(matches!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted { .. })
            ));
        }
        assert_eq!(graph_store.output_commit_reference_count_for_test(1), 3);
        assert_eq!(
            graph_store.published_output_commit(OutputCommitOrdinal(1)),
            Some(&delta)
        );
    }
    graph_store.release(producer).unwrap();
    for (id, causes) in [replace, release, PendingCauseSetId::EMPTY]
        .into_iter()
        .zip(incoming)
    {
        graph_store.replace_normalized(id, causes).unwrap();
    }
    assert_eq!(graph_store.output_commit_reference_count_for_test(1), 0);
    assert_eq!(graph_store.output_commit_reference_count_for_test(2), 2);
    assert!(graph_store
        .published_output_commit(OutputCommitOrdinal(1))
        .is_none());
    assert_eq!(graph_store.allocated_slot_count(), 3);
    assert_eq!(source.output_commit_reference_count_for_test(1), 3);
    assert_eq!(
        source.published_output_commit(OutputCommitOrdinal(1)),
        Some(&delta)
    );
}
