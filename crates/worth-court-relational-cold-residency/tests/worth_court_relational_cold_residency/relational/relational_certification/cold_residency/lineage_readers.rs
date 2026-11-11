//! Public semantic observations retain their actual original cold-page custody.
use super::lineage_sources;
use std::sync::atomic::Ordering;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    change_source::RelationalObservationReadDenial, lineage::RelationalLineageDiscoveryDenial,
    mvcc::RelationalCallerAssociation,
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_medium::{member_placement::ReplacingMemberSource, TestDurableMedium},
    durable_publication::{durable_runtime, memory_runtime, perform},
};

#[test]
fn an_older_lineage_directory_retains_its_cold_pages_after_home_replacement() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let mut placed = None;
    let fixture = lineage_sources::reopened_with_source(execution, |producer| {
        let source = ReplacingMemberSource::new(producer);
        placed = Some(source.clone());
        source
    });
    let source = placed.unwrap();
    // The wrapper preserves its producer's original selected-read retirement.
    use worth_relational::facade::durability::{
        RelationalDurableColdSource, RelationalDurableMemberRetirementObservation,
    };
    let mut original = source
        .checkpoint_selected_read_protection(execution)
        .unwrap();
    source
        .validate_member_custody_retirement(original.as_ref())
        .unwrap();
    assert!(matches!(
        source.advance_member_custody_retirement(execution, original.as_mut()),
        RelationalDurableMemberRetirementObservation::Complete
    ));

    let node_member = fixture
        .directory
        .lineage
        .iter()
        .find(|page| (page.first..=page.last).contains(&fixture.seed.0))
        .unwrap()
        .member;
    source.replace_selected_placement();
    let node = fixture
        .runtime
        .immutable_lineage_node(execution, fixture.seed)
        .unwrap()
        .unwrap();
    assert_eq!(node.node().entity_id(), fixture.original);
    assert!(source.has_owner(node_member));
    drop(node);
    drop(fixture);
    assert!(
        source.owners_released(),
        "the last owner releases placement"
    );
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "Court harness contention: eight original independent OS-thread readers, plus the caller and two registry threads; no computation pool"
)]
fn concurrent_lineage_history_reads_share_one_actual_event_member_load() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let fixture = lineage_sources::reopened(execution);
    let member = fixture
        .directory
        .lineage_events
        .iter()
        .find(|page| {
            page.event_ids
                .iter()
                .any(|id| fixture.expected.traversed_event_ids.contains(id))
        })
        .unwrap()
        .member;
    let start = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let handles = (0..8).map(|_| scope.spawn(|| {
            let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
            let request = &ExecutionRequest::serial(&host);
            start.wait();
            let result = fixture.runtime.discover_authored_lineage_history(request,
                &fixture.runtime.main_branch_identity(), fixture.seed).unwrap();
            assert_eq!(result.digest_basis(), fixture.expected.digest_basis());
        })).collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }
    });
    let reads = fixture.source.member_reads.lock().unwrap();
    assert_eq!(
        reads.iter().filter(|read| **read == member).count(),
        1,
        "one serialized cold load serves the simultaneous demands"
    );
    assert!(
        reads.iter().all(|read| fixture
            .directory
            .lineage_events
            .iter()
            .any(|page| page.member == *read)),
        "lineage traversal reads no region, symbol or node payload"
    );
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "Court harness contention: eight original independent OS-thread readers, plus the caller and two registry threads; no computation pool"
)]
fn simultaneous_lineage_point_reads_share_one_page_load() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let fixture = super::lineage_node_source::reopened(execution);
    // The public Head-only metadata read replaces the private partition warmup.
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    fixture
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap()
        .read_exact_entity_state(execution, fixture.expected.entity_id(), |metadata, _| {
            metadata.entity_id
        })
        .unwrap()
        .unwrap();
    let start = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let workers = (0..8).map(|_| scope.spawn(|| {
            let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
            let request = &ExecutionRequest::serial(&host);
            start.wait();
            let (_, basis) = fixture.runtime.observe_branch(&fixture.runtime.main_branch_identity()).unwrap();
            let node = fixture.runtime.lineage_node_at_observation(request, &basis.observation(), fixture.expected.entity_id())
                .unwrap().unwrap();
            assert_eq!(node.node(), &fixture.expected);
        })).collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
    });
    assert_eq!(
        fixture
            .source
            .member_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|locator| **locator == fixture.page.member)
            .count(),
        1,
        "one cold page load serves all concurrent point operations"
    );
}

#[test]
fn lineage_node_observation_refuses_foreign_root_before_member_reads() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let fixture = super::lineage_node_source::reopened(request);
    let foreign = memory_runtime();
    let (_, basis) = foreign
        .observe_branch(&foreign.main_branch_identity())
        .unwrap();
    assert!(matches!(
        fixture.runtime.lineage_node_at_observation(
            request,
            &basis.observation(),
            fixture.expected.entity_id()
        ),
        Err(RelationalObservationReadDenial::ForeignObservation)
    ));
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), 0);
}

#[test]
fn authored_lineage_discovery_refuses_original_pending_effect_and_continues_after_settlement() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let association =
        RelationalCallerAssociation::from_bytes(b"pending-lineage-discovery").unwrap();
    let performed = perform(request, &runtime, &association, "pending-lineage-record");
    let commit = performed.canonical_commit().commit.commit_id;
    let pending_entity = performed
        .canonical_commit()
        .patch
        .authoritative_record_patches
        .iter()
        .find_map(|patch| match &patch.target {
            worth_relational::facade::transactions::RecordRef::Entity(entity) => Some(*entity),
            worth_relational::facade::transactions::RecordRef::Relation(_) => None,
        })
        .unwrap();
    let before = medium.canonical_journal_reads();
    assert!(
        matches!(runtime.discover_authored_record_history(request, &runtime.main_branch_identity(),
        pending_entity),
        Err(RelationalLineageDiscoveryDenial::PerformedPublicationRequiresSettlement(actual)) if actual == commit)
    );
    assert_eq!(medium.canonical_journal_reads(), before);
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let entity =
        worth_testkit_relational::aspect_storage::publications::changed_entities(&committed)[0];
    let result = runtime
        .discover_authored_record_history(request, &runtime.main_branch_identity(), entity)
        .unwrap()
        .unwrap();
    assert!(!result.resolved.is_empty());
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn lineage_node_observation_demands_its_head_and_node_without_opening_history() {
    use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
    use worth_testkit_relational::durable_medium::immutable_members::ColdRead;
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let fixture = super::lineage_node_source::reopened(request);
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let receipt = fixture
        .runtime
        .history()
        .branch_head_for_observation(&basis.observation())
        .unwrap()
        .unwrap();
    let root = fixture
        .directory
        .roots
        .iter()
        .find(|root| root.commit_id == receipt.commit_id)
        .unwrap();
    let region = root
        .regions
        .iter()
        .find(|region| region.partition_id == fixture.expected.entity_id().partition_id)
        .unwrap();
    let histories = fixture
        .directory
        .mirror
        .iter()
        .chain(&root.regions)
        .map(|region| region.history)
        .collect::<Vec<_>>();
    fixture.source.failures.lock().unwrap().extend(
        histories
            .iter()
            .map(|history| (ColdRead::Member(*history), Cause::SourceUnavailable)),
    );
    let node = fixture
        .runtime
        .lineage_node_at_observation(request, &basis.observation(), fixture.expected.entity_id())
        .unwrap()
        .unwrap();
    assert_eq!(node.node(), &fixture.expected);
    let reads = fixture.source.member_reads.lock().unwrap();
    assert!(reads.contains(&region.head));
    assert!(reads.contains(&fixture.page.member));
    assert!(histories.iter().all(|history| !reads.contains(history)));
}
