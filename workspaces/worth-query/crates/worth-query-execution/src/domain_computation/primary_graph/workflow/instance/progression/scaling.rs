use super::*;
use worth_relational::facade::identity::PartitionId;

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(7), slot, 1)
}

fn populated(count: u64) -> WorkflowInstanceProgress {
    let mut progress = WorkflowInstanceProgress {
        head: entity(count + 1),
        next_occurrence: count,
        retry_counts: OrdMap::new(),
        path_depth: 0,
        path: OrdMap::new(),
        back_blocked_by_operation: false,
        back_edge_iterations: OrdMap::new(),
        latest_transitions: OrdMap::new(),
        latest_transition_identities: OrdMap::new(),
        latest_assessment_evidence: OrdMap::new(),
    };
    for occurrence in 0..count {
        let node = entity(occurrence);
        progress
            .retry_counts
            .insert((node, ApplicationWorkflowControlOutcome::Completed), 1);
        progress.retain_observation(WorkflowTransitionProgressObservation::new(
            WorkflowTransitionLocator::new(
                entity(count + occurrence),
                SettledWorkflowTransition::new(
                    node,
                    occurrence,
                    ApplicationWorkflowControlOutcome::Completed,
                    None,
                ),
            ),
            Some(entity(count * 2 + occurrence)),
        ));
    }
    progress
}

fn update_last(progress: &mut WorkflowInstanceProgress, count: u64) {
    let node = entity(count - 1);
    progress
        .retry_counts
        .insert((node, ApplicationWorkflowControlOutcome::Completed), 2);
    progress.retain_observation(WorkflowTransitionProgressObservation::new(
        WorkflowTransitionLocator::new(
            entity(count * 3),
            SettledWorkflowTransition::new(
                node,
                count,
                ApplicationWorkflowControlOutcome::Completed,
                None,
            ),
        ),
        Some(entity(count * 3 + 1)),
    ));
}

#[test]
fn progress_snapshots_share_maps_and_preserve_isolated_values() {
    for count in [100, 1_000, 10_000] {
        let source = populated(count);
        let mut advanced = source.clone();
        assert!(source.retry_counts.ptr_eq(&advanced.retry_counts));
        assert!(source
            .latest_transitions
            .ptr_eq(&advanced.latest_transitions));
        assert!(source
            .latest_assessment_evidence
            .ptr_eq(&advanced.latest_assessment_evidence));
        update_last(&mut advanced, count);
        assert_eq!(
            source
                .latest_transition(entity(count - 1))
                .unwrap()
                .settlement()
                .occurrence(),
            count - 1
        );
        assert_eq!(
            advanced
                .latest_transition(entity(count - 1))
                .unwrap()
                .settlement()
                .occurrence(),
            count
        );
        assert_ne!(source, advanced);
        let mut duplicate = source.clone();
        update_last(&mut duplicate, count);
        assert_eq!(advanced, duplicate);
    }
}

#[test]
fn ten_thousand_back_settlements_keep_warm_navigation_state_bounded() {
    let first = entity(1);
    let second = entity(2);
    let mut progress = populated(0);
    for cycle in 0..10_000_u64 {
        progress
            .advance_with(
                SettledWorkflowTransition::new(
                    first,
                    cycle * 2,
                    ApplicationWorkflowControlOutcome::Completed,
                    None,
                ),
                &mut |_, _, _| Ok(second),
            )
            .expect("forward settlement is contiguous");
        progress
            .advance_with(
                SettledWorkflowTransition::new(
                    second,
                    cycle * 2 + 1,
                    ApplicationWorkflowControlOutcome::NavigatedBack,
                    None,
                ),
                &mut |_, _, _| panic!("Back derives its predecessor without forward selection"),
            )
            .expect("Back settlement is contiguous");
    }
    assert_eq!(progress.head(), first);
    assert_eq!(progress.next_occurrence(), 20_000);
    assert_eq!(progress.path_depth, 0);
    assert!(progress.path.is_empty());
    assert_eq!(progress.back_edge_iterations.len(), 1);
    assert_eq!(
        progress.back_edge_iterations.get(&(second, first)),
        Some(&10_000)
    );
    let snapshot = progress.clone();
    assert!(snapshot
        .back_edge_iterations
        .ptr_eq(&progress.back_edge_iterations));
}

#[cfg(feature = "allocation-probes")]
#[test]
fn progress_snapshot_and_update_allocation_bounds() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("isolated_progress_allocation_probe")
        .arg("--test-threads=1")
        .arg("--nocapture")
        .env("WORTH_QUERY_PROGRESS_ALLOCATION_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    print!("{}", String::from_utf8_lossy(&output.stdout));
}

#[cfg(feature = "allocation-probes")]
#[test]
fn isolated_progress_allocation_probe() {
    if std::env::var_os("WORTH_QUERY_PROGRESS_ALLOCATION_PROBE").is_none() {
        return;
    }
    use stats_alloc::{Region, INSTRUMENTED_SYSTEM};
    for count in [100, 1_000, 10_000] {
        let construction = Region::new(&INSTRUMENTED_SYSTEM);
        let source = populated(count);
        let allocation = construction.change();
        let live = allocation.bytes_allocated - allocation.bytes_deallocated;
        assert!(
            live > 0,
            "allocation probe requires the instrumented allocator"
        );
        assert!(
            source.retained_charge_bytes() >= live,
            "charge {} below live {live}",
            source.retained_charge_bytes()
        );
        let snapshot = Region::new(&INSTRUMENTED_SYSTEM);
        let mut advanced = source.clone();
        let cloned = snapshot.change();
        assert_eq!(
            cloned.allocations, 0,
            "snapshot must share its three maps at {count}"
        );
        assert_eq!(cloned.bytes_allocated, 0);
        let update = Region::new(&INSTRUMENTED_SYSTEM);
        update_last(&mut advanced, count);
        let changed = update.change();
        // Three 64-pair trees, each with at most four copied levels at 10k.
        // This bounds real allocations; a whole-map clone exceeds it at 1k.
        assert!(changed.allocations <= 12, "{count}: {changed:?}");
        assert!(changed.bytes_allocated <= 100_000, "{count}: {changed:?}");
        let mut duplicate = source.clone();
        update_last(&mut duplicate, count);
        let comparison = Region::new(&INSTRUMENTED_SYSTEM);
        assert_eq!(advanced, duplicate);
        let compared = comparison.change();
        assert!(compared.bytes_allocated <= 16_384, "{count}: {compared:?}");
        println!("progress entries={count}: snapshot_bytes={}, update_allocations={}, update_bytes={}, comparison_bytes={}, live_bytes={live}, charged_bytes={}", cloned.bytes_allocated, changed.allocations, changed.bytes_allocated, compared.bytes_allocated, source.retained_charge_bytes());
    }
}
