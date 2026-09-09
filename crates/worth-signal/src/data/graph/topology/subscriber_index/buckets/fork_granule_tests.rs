use std::env;
use std::process::Command;

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::{IndexedSubscriptionMembership, ReverseSubscriptionIndex};
use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::{
    DetailTokenId, InternedPartitionSubscription, PartitionMatchMode, PartitionTokenId,
};

const TEST_NAME: &str = "data::graph::topology::subscriber_index::buckets::fork_granule_tests::unrelated_replace_does_not_clone_an_inherited_changed_consumer_payload";
const CHILD_PROCESS: &str = "WORTH_SIGNAL_SUBSCRIBER_FORK_GRANULE_CHILD";
const CHILD_COMPLETION: &str = "subscriber fork-granule child completed";

// Two persistent-map path detachments plus one immutable payload handle are the
// honest physical granule. These ceilings leave room for im's bounded tree
// nodes while remaining independent of the inherited consumer's fan-in.
const MAX_UNRELATED_REPLACE_ALLOCATIONS: usize = 128;
const MAX_UNRELATED_REPLACE_BYTES: usize = 128 * 1_024;

#[test]
fn unrelated_replace_does_not_clone_an_inherited_changed_consumer_payload() {
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST_NAME)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD_PROCESS, "1")
            .output()
            .expect("isolated subscriber fork-granule probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        print!("{stdout}");
        eprint!("{stderr}");
        assert!(
            output.status.success(),
            "isolated subscriber fork-granule probe failed"
        );
        assert!(
            stdout.contains(CHILD_COMPLETION),
            "exact child selector did not execute the complete fork-granule probe"
        );
        return;
    }

    let producer = NodeId::new(0, 0);
    let changed_consumer = NodeId::new(1, 0);
    let unrelated_consumer = NodeId::new(2, 0);
    let aspect = Aspect::new(1);
    let source_scope = detail_scope(99, 1);
    let unrelated_scope = detail_scope(8, 1);
    let mut samples = Vec::new();

    for membership_count in [64_u32, 4_096, 65_536] {
        let mut source = ReverseSubscriptionIndex::default();
        source.replace_consumer(
            changed_consumer,
            vec![membership(producer, aspect, source_scope)],
        );

        let mut changed = source.fork_persistent();
        changed.replace_consumer(
            changed_consumer,
            (1..=membership_count)
                .map(|detail| membership(producer, aspect, detail_scope(7, detail)))
                .collect(),
        );
        let sibling = changed.fork_persistent();
        let mut destination = changed.fork_persistent();
        assert!(changed.shares_storage_with(&sibling));
        assert!(changed.shares_storage_with(&destination));

        let unrelated_memberships = vec![membership(producer, aspect, unrelated_scope)];
        let region = Region::new(&INSTRUMENTED_SYSTEM);
        destination.replace_consumer(unrelated_consumer, unrelated_memberships);
        let allocation = region.change();
        println!(
            "subscriber fork granule memberships={membership_count} calls={} bytes={}",
            allocation.allocations, allocation.bytes_allocated
        );
        samples.push((
            membership_count,
            allocation.allocations,
            allocation.bytes_allocated,
        ));

        assert_scope(&source, producer, aspect, source_scope, &[changed_consumer]);
        assert_scope(&source, producer, aspect, detail_scope(7, 1), &[]);
        for preserved in [&changed, &sibling] {
            assert_scope(
                preserved,
                producer,
                aspect,
                detail_scope(7, membership_count),
                &[changed_consumer],
            );
            assert_scope(preserved, producer, aspect, unrelated_scope, &[]);
        }
        assert_scope(
            &destination,
            producer,
            aspect,
            detail_scope(7, membership_count),
            &[changed_consumer],
        );
        assert_scope(
            &destination,
            producer,
            aspect,
            unrelated_scope,
            &[unrelated_consumer],
        );

        verify_changed_consumer_lifecycle(
            &mut destination,
            &changed,
            producer,
            aspect,
            changed_consumer,
        );
    }

    for (membership_count, calls, bytes) in samples {
        assert!(
            calls <= MAX_UNRELATED_REPLACE_ALLOCATIONS,
            "unrelated replace cloned inherited payload at fan-in {membership_count}: {calls} calls exceeds {MAX_UNRELATED_REPLACE_ALLOCATIONS}"
        );
        assert!(
            bytes <= MAX_UNRELATED_REPLACE_BYTES,
            "unrelated replace cloned inherited payload at fan-in {membership_count}: {bytes} bytes exceeds {MAX_UNRELATED_REPLACE_BYTES}"
        );
    }
    println!("{CHILD_COMPLETION}");
}

fn verify_changed_consumer_lifecycle(
    destination: &mut ReverseSubscriptionIndex,
    sibling: &ReverseSubscriptionIndex,
    producer: NodeId,
    aspect: Aspect,
    consumer: NodeId,
) {
    let inherited_scope = detail_scope(7, 1);
    let replacement_scope = detail_scope(9, 1);
    destination.replace_consumer(
        consumer,
        vec![membership(producer, aspect, replacement_scope)],
    );
    assert_scope(destination, producer, aspect, inherited_scope, &[]);
    assert_scope(
        destination,
        producer,
        aspect,
        replacement_scope,
        &[consumer],
    );
    assert_scope(sibling, producer, aspect, inherited_scope, &[consumer]);

    destination.replace_consumer(consumer, Vec::new());
    assert_scope(destination, producer, aspect, replacement_scope, &[]);

    let readmitted_scopes = [detail_scope(10, 1), detail_scope(10, 2)];
    destination.replace_consumer(
        consumer,
        readmitted_scopes
            .into_iter()
            .map(|scope| membership(producer, aspect, scope))
            .collect(),
    );
    for scope in readmitted_scopes {
        assert_scope(destination, producer, aspect, scope, &[consumer]);
    }

    let reconstructed = destination.operational_clone();
    for scope in [
        inherited_scope,
        replacement_scope,
        readmitted_scopes[0],
        readmitted_scopes[1],
    ] {
        assert_eq!(
            reconstructed
                .query_scope(
                    producer,
                    aspect,
                    scope,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap(),
            destination
                .query_scope(
                    producer,
                    aspect,
                    scope,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap(),
            "operational clone must deeply materialize exact overlay membership truth"
        );
    }
}

fn membership(
    producer: NodeId,
    aspect: Aspect,
    scope: InternedPartitionSubscription,
) -> IndexedSubscriptionMembership {
    IndexedSubscriptionMembership::from_edge(producer, aspect, Some(scope))
        .expect("production membership construction accepts an interned detail scope")
}

fn detail_scope(partition: u32, detail: u32) -> InternedPartitionSubscription {
    InternedPartitionSubscription {
        partition: PartitionTokenId(partition),
        detail: Some(DetailTokenId(detail)),
        match_mode: PartitionMatchMode::PartitionAndDetail,
    }
}

fn assert_scope(
    index: &ReverseSubscriptionIndex,
    producer: NodeId,
    aspect: Aspect,
    scope: InternedPartitionSubscription,
    expected: &[NodeId],
) {
    assert_eq!(
        index
            .query_scope(
                producer,
                aspect,
                scope,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary
            )
            .unwrap()
            .candidates,
        expected,
        "scope {scope:?} candidates"
    );
}
