use std::env;
use std::hint::black_box;
use std::process::Command;

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::{IndexedSubscriptionMembership, ReverseSubscriptionIndex};
use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::{
    DetailTokenId, InternedPartitionSubscription, PartitionMatchMode, PartitionTokenId,
};

const TEST_NAME: &str = "data::graph::topology::subscriber_index::buckets::fork_cost_tests::single_membership_first_write_is_bounded_by_nested_persistent_granule";

#[test]
fn single_membership_first_write_is_bounded_by_nested_persistent_granule() {
    const CHILD_PROCESS: &str = "WORTH_SIGNAL_SUBSCRIBER_FORK_COST_CHILD";
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST_NAME)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD_PROCESS, "1")
            .output()
            .expect("isolated subscriber allocation probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{stdout}");
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        assert!(
            output.status.success(),
            "isolated subscriber allocation probe failed"
        );
        assert!(
            stdout.contains(TEST_NAME) && stdout.contains("test result: ok. 1 passed; 0 failed;"),
            "isolated subscriber probe must execute exactly one named test"
        );
        return;
    }

    let producer = NodeId::new(0, 0);
    let aspect = Aspect::new(1);
    let mut samples = Vec::new();
    let mut eager_copy_bytes = None;

    for consumer_count in [64_u32, 4_096, 65_536] {
        let mut source = ReverseSubscriptionIndex::default();
        for index in 0..consumer_count {
            let scope = if index % 2 == 0 {
                InternedPartitionSubscription {
                    partition: PartitionTokenId(index),
                    detail: None,
                    match_mode: PartitionMatchMode::WholePartition,
                }
            } else {
                InternedPartitionSubscription {
                    partition: PartitionTokenId(index),
                    detail: Some(DetailTokenId(index)),
                    match_mode: PartitionMatchMode::PartitionAndDetail,
                }
            };
            let membership =
                IndexedSubscriptionMembership::from_edge(producer, aspect, Some(scope))
                    .expect("interned scoped membership is indexable");
            source.replace_consumer(NodeId::new(index + 1, 0), vec![membership]);
        }
        let removed = NodeId::new(consumer_count, 0);
        let removed_scope = InternedPartitionSubscription {
            partition: PartitionTokenId(consumer_count - 1),
            detail: Some(DetailTokenId(consumer_count - 1)),
            match_mode: PartitionMatchMode::PartitionAndDetail,
        };
        let removed_membership =
            IndexedSubscriptionMembership::from_edge(producer, aspect, Some(removed_scope))
                .expect("removed scoped membership is indexable");
        let mut fork = source.fork_persistent();
        assert!(
            source.shares_storage_with(&fork),
            "owner fork must share the exact immutable index storage"
        );
        let logical_clone = fork.clone();
        assert!(
            fork.shares_storage_with(&logical_clone),
            "logical clone must retain fork roots at scale {consumer_count}"
        );

        if consumer_count == 65_536 {
            let eager_region = Region::new(&INSTRUMENTED_SYSTEM);
            black_box(source.operational_clone());
            eager_copy_bytes = Some(eager_region.change().bytes_allocated);
        }

        let region = Region::new(&INSTRUMENTED_SYSTEM);
        fork.remove_consumer(removed);
        let allocation = region.change();
        samples.push((
            consumer_count,
            allocation.allocations,
            allocation.bytes_allocated,
        ));

        assert_eq!(
            source
                .query_whole_aspect(
                    producer,
                    aspect,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates
                .len(),
            consumer_count as usize,
            "source membership must remain independent"
        );
        assert_eq!(
            fork.query_whole_aspect(
                producer,
                aspect,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary
            )
            .unwrap()
            .candidates
            .len(),
            consumer_count as usize - 1,
            "fork must observe only its changed membership"
        );
        assert!(
            !fork
                .query_scope(
                    producer,
                    aspect,
                    removed_scope,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates
                .contains(&removed),
            "removed base membership must stay retired in the fork"
        );
        let reconstructed = fork.operational_clone();
        assert_eq!(
            reconstructed
                .query_whole_aspect(
                    producer,
                    aspect,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap(),
            fork.query_whole_aspect(
                producer,
                aspect,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary
            )
            .unwrap(),
            "operational reconstruction must preserve the fork overlay"
        );

        fork.replace_consumer(removed, vec![removed_membership]);
        assert!(
            fork.query_scope(
                producer,
                aspect,
                removed_scope,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary
            )
            .unwrap()
            .candidates
            .contains(&removed),
            "readmitting an inherited membership must cancel its retirement"
        );
        assert_eq!(
            source
                .query_whole_aspect(
                    producer,
                    aspect,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates
                .len(),
            consumer_count as usize,
            "fork readmission must not mutate the live source"
        );
    }

    let minimum_calls = samples.iter().map(|(_, calls, _)| *calls).min().unwrap();
    let minimum_bytes = samples.iter().map(|(_, _, bytes)| *bytes).min().unwrap();
    for (consumer_count, calls, bytes) in &samples {
        assert!(
            *calls <= minimum_calls + 64,
            "nested first-write allocation calls slope with {consumer_count} consumers: {calls} vs {minimum_calls}"
        );
        assert!(
            *bytes <= minimum_bytes + 64 * 1_024,
            "nested first-write bytes slope with {consumer_count} consumers: {bytes} vs {minimum_bytes}"
        );
    }
    let maximum_first_write_bytes = samples.iter().map(|(_, _, bytes)| *bytes).max().unwrap();
    assert!(
        eager_copy_bytes.expect("largest sensitivity sample exists")
            > maximum_first_write_bytes.saturating_mul(8),
        "probe must distinguish a whole-index copy from one nested persistent change"
    );
}

#[test]
fn inherited_retirement_queries_visit_only_live_or_changed_members() {
    let producer = NodeId::new(0, 0);
    let aspect = Aspect::new(1);
    let membership = IndexedSubscriptionMembership::from_edge(producer, aspect, None)
        .expect("unscoped membership is indexable");

    for consumer_count in [64_u32, 4_096, 65_536] {
        let mut source = ReverseSubscriptionIndex::default();
        for consumer in 1..=consumer_count {
            source.replace_consumer(NodeId::new(consumer, 0), vec![membership.clone()]);
        }
        let mut fork = source.fork_persistent();
        assert!(source.shares_storage_with(&fork));
        for consumer in 1..consumer_count {
            fork.replace_consumer(NodeId::new(consumer, 0), Vec::new());
        }

        let (query, first_traversal) = fork.query_whole_aspect_with_traversal(producer, aspect);
        let (_, repeated_traversal) = fork.query_whole_aspect_with_traversal(producer, aspect);
        assert_eq!(query.candidates, vec![NodeId::new(consumer_count, 0)]);
        assert!(
            first_traversal.base_members <= 2,
            "scale {consumer_count}: {first_traversal:?}"
        );
        assert!(
            first_traversal.range_seeks <= 2,
            "scale {consumer_count}: {first_traversal:?}"
        );
        assert_eq!(repeated_traversal, first_traversal);
        assert_eq!(
            source
                .query_whole_aspect(
                    producer,
                    aspect,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates
                .len(),
            consumer_count as usize,
            "destination retirement must preserve the live source"
        );

        fork.replace_consumer(NodeId::new(1, 0), vec![membership.clone()]);
        let (readmitted, readmission_traversal) =
            fork.query_whole_aspect_with_traversal(producer, aspect);
        assert_eq!(
            readmitted.candidates,
            vec![NodeId::new(1, 0), NodeId::new(consumer_count, 0)]
        );
        assert!(
            readmission_traversal.base_members <= 3,
            "scale {consumer_count}: {readmission_traversal:?}"
        );
        assert_eq!(
            fork.operational_clone()
                .query_whole_aspect(
                    producer,
                    aspect,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap(),
            readmitted
        );
    }
}

#[test]
fn ordinary_queries_stream_unretired_base_without_range_reseeks() {
    let producer = NodeId::new(0, 0);
    let aspect = Aspect::new(1);
    let membership = IndexedSubscriptionMembership::from_edge(producer, aspect, None)
        .expect("unscoped membership is indexable");

    for consumer_count in [64_u32, 4_096, 65_536] {
        let mut source = ReverseSubscriptionIndex::default();
        for consumer in 1..=consumer_count {
            source.replace_consumer(NodeId::new(consumer, 0), vec![membership.clone()]);
        }
        let mut fork = source.fork_persistent();
        let added = NodeId::new(consumer_count + 1, 0);
        fork.replace_consumer(added, vec![membership.clone()]);

        let (query, traversal) = fork.query_whole_aspect_with_traversal(producer, aspect);
        assert_eq!(query.candidates.len(), consumer_count as usize + 1);
        assert_eq!(traversal.base_members, consumer_count as usize);
        assert_eq!(
            traversal.range_seeks, 0,
            "unretired base traversal must stay on the native linear iterator"
        );
        assert_eq!(
            source
                .query_whole_aspect(
                    producer,
                    aspect,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates
                .len(),
            consumer_count as usize
        );
    }
}
