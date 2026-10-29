//! Batch B locality through mounted delta production and atlas preplanning.

use worth_ui_host_contract::{
    UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedPresentationProductionCost,
    UiMountedPresentationWorkView,
};

use super::super::work_producer::UiMountedPresentationState;
use super::world::MountedPresentationWorld;
use crate::mounting::qualified_text_test_support::UiQualifiedTextTestFixture;
use crate::native_platform::text_presentation::{
    prepare_mounted_semantic_text, UiMountedEventTimeDpiAuthority, UiNativeTextPresentationPrepared,
};

#[test]
fn batch_b_demand_is_local_and_unplanned_work_never_rasterizes_smoke() {
    let mut baseline = None;
    let fixture = measured_fixture_setup("smoke");
    for (retained, changed_index) in [(1, 0), (32, 16)] {
        let observed = assert_local_preplan(&fixture, retained, changed_index);
        assert_eq!(baseline.get_or_insert(observed), &observed);
    }
}

fn measured_fixture_setup(case: &str) -> UiQualifiedTextTestFixture {
    let started = std::time::Instant::now();
    let fixture = UiQualifiedTextTestFixture::new();
    println!(
        "WORTH_UI_BATCH_B_FIXTURE_TIMING={{\"case\":\"{case}\",\"elapsed_ms\":{}}}",
        started.elapsed().as_millis(),
    );
    fixture
}

fn assert_local_preplan(
    fixture: &UiQualifiedTextTestFixture,
    retained: usize,
    changed_index: usize,
) -> ObservedBatchBPreplanCost {
    let started = std::time::Instant::now();
    let observed = prepare_one_changed_text(fixture, retained, changed_index);
    println!(
        "WORTH_UI_BATCH_B_PREPLAN_TIMING={{\"retained\":{retained},\"changed_index\":{changed_index},\"elapsed_ms\":{}}}",
        started.elapsed().as_millis(),
    );
    assert_eq!(observed.producer_retained_scans, 0);
    assert_eq!(observed.producer_retained_clones, 0);
    assert_eq!(observed.raster_retained_scans, 0);
    assert_eq!(observed.layout_count, 1);
    assert_eq!(observed.paint_spans, 1);
    assert_eq!(observed.demand_batches, 1);
    assert!(observed.demand_records > 0);
    assert_eq!(observed.key_checks, observed.demand_records);
    assert_eq!(observed.rasterized_glyphs, 0);
    assert_eq!(observed.rasterized_texels, 0);
    assert_eq!(observed.produced_bytes, 0);
    assert_eq!(observed.producer_source_instances, 1);
    assert_eq!(observed.producer_commands_considered, 1);
    assert!(observed.layout_visits > 0);
    assert!(observed.demanded_glyphs > 0);
    observed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObservedBatchBPreplanCost {
    layout_count: u32,
    paint_spans: u32,
    demand_batches: u32,
    demand_records: u32,
    layout_visits: u32,
    demanded_glyphs: u32,
    key_checks: u32,
    rasterized_glyphs: u32,
    rasterized_texels: u64,
    produced_bytes: u64,
    producer_source_instances: u64,
    producer_commands_considered: u64,
    producer_retained_scans: u64,
    producer_retained_clones: u64,
    raster_retained_scans: u32,
}

fn prepare_one_changed_text(
    fixture: &UiQualifiedTextTestFixture,
    retained: usize,
    changed_index: usize,
) -> ObservedBatchBPreplanCost {
    let world = MountedPresentationWorld::new();
    let instances = (0..retained)
        .map(|_| UiMountedInstanceIdentity::mint_unbound().unwrap())
        .collect::<Vec<_>>();
    let layout = fixture.layout("WORTH");
    let predecessor = world.text_projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        &instances,
        None,
        layout.view(),
    );
    let successor = world.text_projection(
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        &instances,
        Some(changed_index),
        layout.view(),
    );
    let predecessor_state =
        UiMountedPresentationState::from_projection(&predecessor, world.requirement, None);
    let successor_state = UiMountedPresentationState::from_projection(
        &successor,
        world.requirement,
        Some(predecessor.frame()),
    );
    let lease = super::super::UiMountedPresentationLeaseGate::default()
        .claim()
        .unwrap();
    let work = predecessor_state
        .issue_successor(super::super::work_producer::SuccessorIssueRequest::new(
            &successor_state,
            &[instances[changed_index]],
            &[],
            &lease,
        ))
        .unwrap();
    let producer_cost = match work.view() {
        UiMountedPresentationWorkView::Delta(delta) => delta.production_cost(),
        _ => panic!("one changed text mechanic must issue delta work"),
    };
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(world.requirement).unwrap();
    let prepared = prepare_mounted_semantic_text(work.view(), dpi, &|identity| {
        (identity == layout.identity()).then_some(layout.as_ref())
    })
    .unwrap();
    let crate::native_platform::text_presentation::UiNativeTextPresentationPreparation::Prepared(
        prepared,
    ) = prepared
    else {
        panic!("exact text must produce prepared demand work")
    };
    observed_cost(&prepared, producer_cost)
}

fn observed_cost(
    prepared: &UiNativeTextPresentationPrepared,
    producer: UiMountedPresentationProductionCost,
) -> ObservedBatchBPreplanCost {
    let planning = prepared.planning_inspection().unwrap();
    let demand = prepared.demand_batches()[0].cost().ordinary();
    let raster = prepared.raster_work();
    ObservedBatchBPreplanCost {
        layout_count: u32::try_from(prepared.layout_count()).unwrap(),
        paint_spans: u32::try_from(prepared.paint_span_count()).unwrap(),
        demand_batches: planning.demand_batches(),
        demand_records: planning.demand_records(),
        layout_visits: demand.layout_visits(),
        demanded_glyphs: demand.demanded_glyphs(),
        key_checks: planning.key_checks(),
        rasterized_glyphs: raster.rasterized_glyphs(),
        rasterized_texels: raster.rasterized_texels(),
        produced_bytes: raster.produced_bytes(),
        producer_source_instances: producer.source_instances(),
        producer_commands_considered: producer.commands_considered(),
        producer_retained_scans: producer.retained_command_scans(),
        producer_retained_clones: producer.retained_command_clones(),
        raster_retained_scans: demand.retained_scans(),
    }
}
