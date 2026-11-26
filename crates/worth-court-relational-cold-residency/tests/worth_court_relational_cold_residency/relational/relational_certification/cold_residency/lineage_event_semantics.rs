//! Public event resolutions and noninterference, without a private query-index census.
use super::lineage_sources::reopened;
use worth_relational::facade::identity::LineageId;
#[test]
fn reopened_lineage_resolution_preserves_original_replacement_chain_and_digest() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let observed = fixture
        .runtime
        .discover_authored_lineage_history(
            execution,
            &fixture.runtime.main_branch_identity(),
            fixture.seed,
        )
        .unwrap();
    assert_eq!(observed.resolved, fixture.expected.resolved);
    assert_eq!(
        observed.traversed_event_ids,
        fixture.expected.traversed_event_ids
    );
    assert_eq!(observed.digest_basis(), fixture.expected.digest_basis());
    assert!(
        fixture
            .runtime
            .observe_lineage_residency()
            .resident_event_count
            < 601 / 2,
        "resolution loads only touched event pages"
    );
}

#[test]
fn forked_lineage_queries_preserve_original_resolution_without_reloading_members() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let fork = fixture.runtime.fork().unwrap();
    let first = fixture
        .runtime
        .discover_authored_lineage_history(
            execution,
            &fixture.runtime.main_branch_identity(),
            fixture.seed,
        )
        .unwrap();
    let reads = fixture.source.member_reads.lock().unwrap().len();
    assert_eq!(
        fork.observe_lineage_residency().resident_event_count,
        0,
        "fork starts with no per-runtime admitted events"
    );

    let second = fork
        .discover_authored_lineage_history(execution, &fork.main_branch_identity(), fixture.seed)
        .unwrap();
    assert_eq!(first.digest_basis(), second.digest_basis());
    assert_eq!(second.resolved, fixture.expected.resolved);
    let admitted = fork.observe_lineage_residency().resident_event_count;
    assert!(admitted > 0 && admitted < 601 / 2);
    for runtime in [&fixture.runtime, &fork] {
        let warm = runtime
            .discover_authored_lineage_history(
                execution,
                &runtime.main_branch_identity(),
                fixture.seed,
            )
            .unwrap();
        assert_eq!(warm.digest_basis(), fixture.expected.digest_basis());
    }

    assert_eq!(
        fork.observe_lineage_residency().resident_event_count,
        admitted
    );
    assert_eq!(fixture.source.member_reads.lock().unwrap().len(), reads);
}

#[test]
fn node_range_boundaries_keep_neighboring_event_members_cold() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let seed = LineageId(512);
    let unrelated = fixture
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
    let resolution = fixture
        .runtime
        .discover_authored_lineage_history(execution, &fixture.runtime.main_branch_identity(), seed)
        .unwrap();
    assert_eq!(resolution.resolved, vec![seed]);
    assert!(
        resolution.traversed_event_ids.is_empty(),
        "this producer lineage was never replaced"
    );
    assert!(
        fixture
            .runtime
            .observe_lineage_residency()
            .resident_event_count
            < 601 / 2
    );
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .contains(&unrelated));
}
