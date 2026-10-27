//! Reopened lineage traversal must preserve genuine replacement edges.
use super::cold_lineage_event_fixture::{reopened, request};

#[test]
fn a_reopened_lineage_resolution_follows_only_its_actual_event_pages() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let observed = fixture
        .runtime
        .lineage_access()
        .resolve_historical_lineage(execution, request(fixture.seed))
        .unwrap();
    assert_eq!(observed.resolved, fixture.expected.resolved);
    assert_eq!(
        observed.traversed_event_ids,
        fixture.expected.traversed_event_ids
    );
    assert_eq!(observed.digest_basis(), fixture.expected.digest_basis());
    assert!(
        fixture.runtime.lineage.event_count() < 601 / 2,
        "a lineage query leaves unrelated node ranges' events cold"
    );
}

#[test]
fn a_shared_event_page_is_admitted_once_into_each_forked_lineage_authority() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let fork = fixture.runtime.fork().unwrap();
    let first = fixture
        .runtime
        .lineage_access()
        .resolve_historical_lineage(execution, request(fixture.seed))
        .unwrap();
    let reads = fixture.source.member_reads.lock().unwrap().len();
    assert_eq!(
        fork.lineage.event_count(),
        0,
        "the fork's index has not admitted shared bytes yet"
    );
    let second = fork
        .lineage_access()
        .resolve_historical_lineage(execution, request(fixture.seed))
        .unwrap();
    assert_eq!(first.digest_basis(), second.digest_basis());
    assert_eq!(second.resolved, fixture.expected.resolved);
    assert!(fork.lineage.event_count() < 601 / 2);
    let admitted = fork.lineage.event_count();
    for runtime in [&fixture.runtime, &fork] {
        let warm = runtime
            .lineage_access()
            .resolve_historical_lineage(execution, request(fixture.seed))
            .unwrap();
        assert_eq!(warm.digest_basis(), fixture.expected.digest_basis());
    }
    assert_eq!(
        fork.lineage.event_count(),
        admitted,
        "a warm query never reinstalls its page"
    );
    assert_eq!(fixture.source.member_reads.lock().unwrap().len(), reads);
}

#[test]
fn concurrent_lineage_history_reads_share_one_actual_event_member_load() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let directory = fixture
        .runtime
        .durability
        .checkpoint_selection()
        .snapshot()
        .unwrap();
    let member = directory
        .manifest
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
        let handles = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let host = crate::test_execution_policy::serial_request();
                    let execution = &worth_execution::ExecutionRequest::serial(&host);
                    start.wait();
                    let result = fixture
                        .runtime
                        .lineage_access()
                        .resolve_historical_lineage(execution, request(fixture.seed))
                        .unwrap();
                    assert_eq!(result.digest_basis(), fixture.expected.digest_basis());
                })
            })
            .collect::<Vec<_>>();
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
        reads.iter().all(|read| directory
            .manifest
            .lineage_events
            .iter()
            .any(|page| page.member == *read)),
        "lineage traversal reads no region, symbol or node payload"
    );
}

#[test]
fn node_range_boundaries_keep_neighboring_event_members_cold() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let seed = crate::identity::data::LineageId(512);
    let directory = fixture
        .runtime
        .durability
        .checkpoint_selection()
        .snapshot()
        .unwrap();
    let unrelated = directory
        .manifest
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
        .lineage_access()
        .resolve_historical_lineage(execution, request(seed))
        .unwrap();
    assert_eq!(resolution.resolved, vec![seed]);
    assert!(
        resolution.traversed_event_ids.is_empty(),
        "this producer lineage was never replaced"
    );
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .contains(&unrelated));
    assert!(fixture.runtime.lineage.event_count() < 601 / 2);
}

#[test]
fn native_capture_from_a_fork_uses_shared_resident_event_pages_without_losing_rows() {
    use crate::durability::data::RelationalCheckpointArtifactPolicy;
    use std::collections::BTreeSet;
    use std::sync::atomic::Ordering;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = reopened(execution);
    let fork = fixture.runtime.fork().unwrap();
    fixture
        .runtime
        .lineage
        .demand_event_pages(execution, None, None)
        .unwrap();
    assert_eq!(
        fork.lineage.event_count(),
        0,
        "shared residency does not populate another runtime's index"
    );
    let directory = fork.durability.checkpoint_selection().snapshot().unwrap();
    fork.services
        .symbols
        .demand_symbol_ids(
            execution,
            directory
                .manifest
                .symbols
                .iter()
                .map(|page| crate::symbols::data::Symbol(page.first as u32)),
        )
        .unwrap();
    for page in &directory.manifest.lineage {
        fork.lineage
            .demand_node(execution, crate::identity::data::LineageId(page.first))
            .unwrap();
    }
    fork.acquire_partition_edition()
        .demand_all(execution, &fork.services.symbols)
        .unwrap();
    let (_, basis) = fork.observe_branch(&fork.main_branch_identity()).unwrap();
    basis
        .observation()
        .selected_root()
        .demand_all_full(execution, &fork.services.symbols)
        .unwrap();
    let reads = fixture.source.reads.load(Ordering::Relaxed);
    let native = fork
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[directory.manifest.derived_indexes],
                source: Some(fixture.source.clone()),
            },
        )
        .unwrap();
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let output = rmp_serde::from_slice::<Input>(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let observed = output
        .manifest
        .lineage_events
        .iter()
        .flat_map(|page| page.event_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    let expected = directory
        .manifest
        .lineage_basis
        .canonical_published_event_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed, expected,
        "a no-load native capture must preserve every shared event row"
    );
    assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
    assert_eq!(
        fork.lineage.event_count(),
        0,
        "capture reads resident pages without rewriting the runtime's query index"
    );
}
