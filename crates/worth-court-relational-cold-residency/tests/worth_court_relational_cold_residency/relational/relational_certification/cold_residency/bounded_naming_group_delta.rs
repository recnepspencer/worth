//! Cold-reopen composition court for public #455: a one-member detach from
//! an `M`-member group reopened cold from a `CHK24` checkpoint faults a
//! bounded number of naming pages and publishes the same group lineage frame
//! for `M = 64` and `M = 1280`. The kept group's member delta reads and writes only
//! the detached member's rows (its `GROUP_MEMBERS` and `GROUP_MEMBER_OWNER`
//! rows and the group's header), never the group's untouched members.

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, naming_page_reads, publish_group_partition,
    reopen_named,
};

/// Naming pages faulted, and group frame bytes published, by detaching the
/// last member of a reopened `members`-member Anchor group.
fn detach_after_cold_reopen(members: usize) -> (usize, usize) {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();
    let labels: Vec<String> = (0..members)
        .map(|i| format!("delta-member-{i:04}"))
        .collect();
    let (_, entities, names) = create_named_entities(request, &producer, &main, &labels);
    let group = names
        .iter()
        .enumerate()
        .map(|(i, &name)| match i {
            0 => (name, AggregateMemberRole::Anchor),
            _ => (name, AggregateMemberRole::Ordinary),
        })
        .collect();
    publish_group_partition(request, &producer, &main, entities[0], "whole", vec![group]);
    let (checkpoint, _) = capture_external_checkpoint(request, &producer);
    let source = immutable_source(&checkpoint, &medium);
    let reopened = reopen_named(request, checkpoint, source.clone(), &medium, u64::MAX / 4);
    let before = naming_page_reads(&source);
    let detached = vec![vec![(names[members - 1], AggregateMemberRole::Anchor)]];
    let committed =
        publish_group_partition(request, &reopened, &main, entities[0], "detach", detached);
    let faulted = naming_page_reads(&source) - before;
    (faulted, committed.group_lineage_frame().len())
}

#[test]
fn cold_one_member_detach_faults_and_publishes_independent_of_group_size() {
    let (small_pages, small_frame) = detach_after_cold_reopen(64);
    let (large_pages, large_frame) = detach_after_cold_reopen(1280);
    println!(
        "detach one of 64 after cold reopen: {small_pages} naming pages, frame {small_frame} B"
    );
    println!(
        "detach one of 1280 after cold reopen: {large_pages} naming pages, frame {large_frame} B"
    );
    assert!(small_frame > 0);
    assert_eq!(
        large_frame, small_frame,
        "the frame records the delta alone"
    );
    // The commit probes a fixed set of keys: the group's least members, the
    // anchor's and the detached member's `GROUP_MEMBERS` and owner rows, and
    // the index rows. Spread over more pages, each of those four table ranges
    // costs at most one more page; walking the group's 1280 rows would fault
    // every page of the member and owner tables (five or more each).
    assert!(
        large_pages <= small_pages + 4,
        "faulted pages {small_pages} -> {large_pages}"
    );
}
