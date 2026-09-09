use super::*;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::retained_storage::RetainedStoragePreparation as Work;

#[test]
fn release_admission_is_exact_and_denial_preserves_retained_set() {
    let mut source = CanonicalCauseSetStore::default();
    let cause = ResolvedDependencyCause::new(
        1,
        NodeId::new(1, 0),
        DependencyRevision(1),
        NodeId::new(2, 0),
        crate::data::aspect::Aspect::new(1),
        None,
        0,
        OutputCommitOrdinal(1),
        1,
        Default::default(),
    );
    let id = source.insert([cause]);
    let scope = "retained-scope".repeat(1000);
    let aspect = crate::data::aspect::Aspect::new(1);
    let delta = crate::data::proof::invalidation::output_commit::ProducedAspectDelta::from_committed_result(
        NodeId::new(2, 0), source.reserve_output_commit_ordinal(),
        crate::data::aspect::AspectVersion::zero(),
        crate::data::aspect::AspectVersion::from_updates([(aspect, 1)]),
        crate::data::aspect::AspectMask::from_aspect(aspect),
        &[(aspect, crate::data::output::ChangedRegion::new(scope.as_str()).with_detail(scope.as_str()))],
        &[],
    ).unwrap();
    source.publish_output_commit(delta.clone());
    assert_eq!(
        source.published_output_commit(OutputCommitOrdinal(1)),
        Some(&delta)
    );
    let mut fork = source.fork_persistent();
    let mut measured = Work::new(usize::MAX);
    fork.admit_release_work(id, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let cost = measured.visits();
    assert!(cost >= 2 * scope.len());
    for available in [cost - 1, cost] {
        let mut work = Work::new(cost + 11);
        work.reserve_visits(cost + 11 - available).unwrap();
        let result = fork.admit_release_work(id, &mut EvaluationWork::Conditional(&mut work));
        if available == cost {
            result.unwrap();
            assert_eq!(work.visits(), cost + 11);
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: cost + 11
                })
            );
        }
        assert_eq!(fork.get(id).unwrap(), source.get(id).unwrap());
        assert_eq!(fork.output_commit_reference_count_for_test(1), 1);
        assert_eq!(
            fork.published_output_commit(OutputCommitOrdinal(1)),
            Some(&delta)
        );
    }
    fork.release(id).unwrap();
    assert_eq!(fork.output_commit_reference_count_for_test(1), 0);
    assert_eq!(source.get(id).unwrap().len(), 1);
    assert!(fork.get(id).is_err());
    assert!(fork
        .published_output_commit(OutputCommitOrdinal(1))
        .is_none());
    assert_eq!(
        source.published_output_commit(OutputCommitOrdinal(1)),
        Some(&delta)
    );
}

#[test]
fn conditional_release_denies_missing_generation_metadata_without_repair() {
    let mut store = CanonicalCauseSetStore::default();
    let cause = ResolvedDependencyCause::new(
        1,
        NodeId::new(1, 0),
        DependencyRevision(1),
        NodeId::new(2, 0),
        crate::data::aspect::Aspect::new(1),
        None,
        0,
        OutputCommitOrdinal(1),
        1,
        Default::default(),
    );
    let id = store.insert([cause]);
    store.slot_generations.clear();
    let mut work = Work::new(usize::MAX);
    assert!(store
        .admit_release_work(id, &mut EvaluationWork::Conditional(&mut work))
        .is_err());
    assert_eq!(store.slot_generations.len(), 0);
    assert_eq!(store.sets.len(), 1);
}
