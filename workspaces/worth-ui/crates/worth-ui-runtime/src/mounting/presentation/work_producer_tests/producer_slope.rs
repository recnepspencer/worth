use std::time::{Duration, Instant};
use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedPresentationWorkView,
    UiMountedRgba8,
};

use super::super::work_producer::UiMountedPresentationState;
use super::world::{rect_spec, MountedPresentationWorld};

/// Retained sizes up to this fit the Portal overlay table alone; larger cases
/// mix overlays with qualified text so the fixture stays within every table cap.
const OVERLAY_ONLY_RETAINED: usize = worth_ui_host_contract::UiMountedPortalOverlayTable::MAX_ROWS;

#[test]
fn admitted_sources_leave_only_local_work_inside_delta_issuance() {
    run_locality_cases(&[(1, 0), (32, 16), (2_048, 1_024), (4_096, 2_048)]);
}

#[test]
fn complete_retained_scan_and_clone_mutant_is_rejected_by_the_local_oracle() {
    let (actual, _) = exercise_one_change(32, 16, None, ProducerPath::RetainedScanMutant);
    assert_eq!(actual[4], 32);
    assert_eq!(actual[5], 32);
    assert!(!is_local_cost(actual, 32));
}

fn run_locality_cases(cases: &[(usize, usize)]) {
    let fixture_started = Instant::now();
    let text_layout = cases
        .iter()
        .any(|(retained, _)| *retained > OVERLAY_ONLY_RETAINED)
        .then(|| {
            crate::mounting::qualified_text_test_support::UiQualifiedTextTestFixture::new()
                .layout("WORTH")
        });
    println!(
        "WORTH_UI_PRODUCER_SLOPE_FIXTURE_TIMING={{\"materialized\":{},\"elapsed_ms\":{}}}",
        text_layout.is_some(),
        fixture_started.elapsed().as_millis(),
    );
    for &(retained, changed_index) in cases {
        let (actual, timing) = exercise_one_change(
            retained,
            changed_index,
            text_layout.as_ref().map(|layout| layout.view()),
            ProducerPath::Ordinary,
        );
        assert_eq!(actual, expected_local_cost(retained), "retained={retained}");
        report_timing(retained, changed_index, timing);
    }
}

fn expected_local_cost(retained: usize) -> [u64; 7] {
    [1, 1, 2, 2, 0, 0, u64::try_from(retained * 2).unwrap()]
}

fn is_local_cost(actual: [u64; 7], retained: usize) -> bool {
    actual == expected_local_cost(retained)
}

#[derive(Clone, Copy)]
enum ProducerPath {
    Ordinary,
    RetainedScanMutant,
}

#[derive(Clone, Copy)]
struct FixtureTiming {
    identities: Duration,
    predecessor_projection: Duration,
    successor_projection: Duration,
    retained_state: Duration,
    delta_issuance: Duration,
}

fn exercise_one_change(
    retained: usize,
    changed_index: usize,
    text_layout: Option<worth_ui_host_contract::UiQualifiedTextLayoutView<'_>>,
    path: ProducerPath,
) -> ([u64; 7], FixtureTiming) {
    let identities_started = Instant::now();
    let world = MountedPresentationWorld::new();
    let instances = (0..retained)
        .map(|_| UiMountedInstanceIdentity::mint_unbound().unwrap())
        .collect::<Vec<_>>();
    let identities = identities_started.elapsed();
    let predecessor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let predecessor_started = Instant::now();
    let predecessor = if retained > OVERLAY_ONLY_RETAINED {
        world.mixed_projection(
            predecessor_frame,
            &instances,
            None,
            text_layout.expect("mixed closure case requires a qualified-text fixture"),
        )
    } else {
        world.projection(
            predecessor_frame,
            instances
                .iter()
                .enumerate()
                .map(|(index, instance)| rect_spec(*instance, index as f32 * 40.0)),
        )
    };
    let predecessor_projection = predecessor_started.elapsed();
    let successor_started = Instant::now();
    let successor = if retained > OVERLAY_ONLY_RETAINED {
        world.mixed_projection(
            successor_frame,
            &instances,
            Some(changed_index),
            text_layout.expect("mixed closure case requires a qualified-text fixture"),
        )
    } else {
        world.projection(
            successor_frame,
            instances.iter().enumerate().map(|(index, instance)| {
                let mut spec = rect_spec(*instance, index as f32 * 40.0);
                if index == changed_index {
                    spec.color = UiMountedRgba8::new(242, 204, 96, 255);
                }
                spec
            }),
        )
    };
    let successor_projection = successor_started.elapsed();
    let retained_started = Instant::now();
    let predecessor_state =
        UiMountedPresentationState::from_projection(&predecessor, world.requirement, None);
    let successor_state = UiMountedPresentationState::from_projection(
        &successor,
        world.requirement,
        Some(predecessor.frame()),
    );
    let retained_state = retained_started.elapsed();
    let delta_started = Instant::now();
    let lease = super::super::UiMountedPresentationLeaseGate::default()
        .claim()
        .unwrap();
    let work = match path {
        ProducerPath::Ordinary => predecessor_state.issue_successor(
            super::super::work_producer::SuccessorIssueRequest::new(
                &successor_state,
                std::slice::from_ref(&instances[changed_index]),
                &[],
                &lease,
            ),
        ),
        ProducerPath::RetainedScanMutant => predecessor_state
            .issue_successor_with_complete_retained_scan_mutant(
                super::super::work_producer::SuccessorIssueRequest::new(
                    &successor_state,
                    std::slice::from_ref(&instances[changed_index]),
                    &[],
                    &lease,
                ),
            ),
    }
    .unwrap();
    let delta_issuance = delta_started.elapsed();
    let UiMountedPresentationWorkView::Delta(delta) = work.view() else {
        panic!("one changed command must issue delta work");
    };
    assert_eq!(delta.changes().len(), 1);
    let changed_identity = match &delta.changes()[0] {
        worth_ui_host_contract::UiMountedPaintCommandChange::Insert(command)
        | worth_ui_host_contract::UiMountedPaintCommandChange::Replace {
            successor: command, ..
        } => command.identity(),
        worth_ui_host_contract::UiMountedPaintCommandChange::Remove(identity) => *identity,
    };
    assert_eq!(
        changed_identity.mounted_instance(),
        instances[changed_index]
    );
    let cost = delta.production_cost();
    (
        [
            cost.source_instances(),
            cost.commands_considered(),
            cost.command_index_lookups(),
            cost.order_lookups(),
            cost.retained_command_scans(),
            cost.retained_command_clones(),
            cost.projection_rows_materialized(),
        ],
        FixtureTiming {
            identities,
            predecessor_projection,
            successor_projection,
            retained_state,
            delta_issuance,
        },
    )
}

fn report_timing(retained: usize, changed_index: usize, timing: FixtureTiming) {
    println!(
        "WORTH_UI_PRODUCER_SLOPE_TIMING={{\"retained\":{retained},\"changed_index\":{changed_index},\"identities_ms\":{},\"predecessor_projection_ms\":{},\"successor_projection_ms\":{},\"retained_state_ms\":{},\"delta_issuance_ms\":{}}}",
        timing.identities.as_millis(),
        timing.predecessor_projection.as_millis(),
        timing.successor_projection.as_millis(),
        timing.retained_state.as_millis(),
        timing.delta_issuance.as_millis(),
    );
}
