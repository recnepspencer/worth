mod appearance_scale;
mod capture;
mod lifecycle;
mod mixed_carrier;
mod oracle;
mod world;

use oracle::{adjudicate, expectation, ordered_pixel, OracleDenial};
use world::{produce_maximum_overlap, MountedPresentationWorld};

#[test]
fn mixed_carrier_successors_are_local_in_ordinary_smoke() {
    assert_mixed_carrier(mixed_carrier::SMOKE);
}

#[test]
#[ignore = "closure courtroom: mounts the full 2,048 rectangle + 2,048 text public world"]
fn mixed_carrier_successors_are_local_at_the_4096_command_ceiling() {
    let production = assert_mixed_carrier(mixed_carrier::CLOSURE);
    assert_collection_row_correlation(&production.initial, 1_359);
}

fn assert_mixed_carrier(
    profile: mixed_carrier::MixedCarrierFixtureProfile,
) -> mixed_carrier::MixedCarrierProduction {
    let recorder = worth_ui_host_headless::WorthUiHeadlessRecorder::with_viewport_extent(
        worth_ui_host_headless::UiHeadlessRecorderCapacity::new(1, 8, 16_384),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let production = mixed_carrier::produce(recorder, profile);
    assert_eq!(
        world::ordered_surfaces(&production.appearance_baseline).len(),
        profile.rectangle_count
    );
    assert_eq!(
        world::ordered_surfaces(&production.initial).len(),
        profile.scalar_instance_count
    );
    assert_eq!(production.initial.semantic_text().len(), profile.text_count);
    assert_eq!(production.initial.nodes().len(), profile.rectangle_count);
    assert_eq!(
        world::ordered_surfaces(&production.text_replacement).len(),
        1
    );
    assert_eq!(
        production.text_replacement.semantic_text().len(),
        profile.text_count
    );
    assert_eq!(
        production.text_replacement.nodes().len(),
        profile.rectangle_count
    );
    assert_eq!(
        world::ordered_surfaces(&production.rectangle_removal).len(),
        0
    );
    assert_eq!(
        production.rectangle_removal.semantic_text().len(),
        profile.text_count
    );
    assert_eq!(
        production.rectangle_removal.nodes().len(),
        profile.rectangle_count - 1
    );
    assert_eq!(
        world::ordered_surfaces(&production.rectangle_insertion).len(),
        1
    );
    assert_eq!(
        production.rectangle_insertion.semantic_text().len(),
        profile.text_count
    );
    assert_eq!(
        production.rectangle_insertion.nodes().len(),
        profile.rectangle_count
    );
    assert_eq!(
        mixed_carrier::collection_value_count(&production.initial, "Ready"),
        1
    );
    assert_eq!(
        mixed_carrier::collection_value_count(
            &production.text_replacement,
            &mixed_carrier::replacement_value(profile, profile.collection_rows - 1),
        ),
        1
    );
    assert_eq!(
        mixed_carrier::collection_value_count(
            &production.text_replacement,
            &mixed_carrier::initial_value(profile, profile.collection_rows - 1),
        ),
        0
    );
    assert_eq!(
        mixed_carrier::collection_value_count(&production.text_replacement, "Ready"),
        1
    );
    assert_eq!(
        mixed_carrier::text_bytes(&production.initial),
        profile.text_bytes
    );
    assert_eq!(
        mixed_carrier::text_bytes(&production.text_replacement),
        profile.text_bytes
    );
    assert_text_successor_cost(production.costs[1]);
    for cost in &production.costs[2..4] {
        assert_appearance_successor_cost(*cost);
    }
    assert_zero_successor_cost(production.costs[4]);
    assert_adapter_delta(&production.adapter_costs[1..4]);
    assert_eq!(production.adapter_costs[4], Default::default());
    production
}

fn assert_collection_row_correlation(
    transcript: &worth_ui_host_headless::UiHeadlessMountedFrameTranscript,
    expected_rows: usize,
) {
    let collection = transcript
        .semantic_text()
        .iter()
        .filter(|mechanic| mechanic.collection_row().is_some())
        .collect::<Vec<_>>();
    assert_eq!(collection.len(), expected_rows);
    assert!(collection.iter().all(|mechanic| {
        mechanic.slot()
            == worth_ui_host_contract::UiSemanticTextSlot::CollectionValue {
                selected_field_ordinal: 0,
            }
    }));
    let correlations = collection
        .iter()
        .map(|mechanic| mechanic.collection_row().unwrap().correlation_digest())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(correlations.len(), expected_rows);
}

fn assert_adapter_delta(costs: &[worth_ui_host_contract::UiHostPresentationCostReport]) {
    let expected = [(4, 4), (4, 3), (5, 3)];
    assert_eq!(costs.len(), expected.len());
    for (cost, (translated_rows, delta_rows)) in costs.iter().zip(expected) {
        assert_eq!(cost.translated_rows(), translated_rows);
        assert_eq!(cost.delta_rows_carried(), delta_rows);
    }
}

fn assert_text_successor_cost(cost: worth_ui_host_contract::UiMountedPresentationProductionCost) {
    assert_eq!(cost.source_instances(), 1);
    assert_eq!(cost.commands_considered(), 1);
    assert_eq!(cost.command_index_lookups(), 2);
    assert_eq!(cost.order_lookups(), 2);
    assert_eq!(cost.retained_command_scans(), 0);
    assert_eq!(cost.retained_command_clones(), 0);
    assert_eq!(cost.projection_rows_materialized(), 0);
}

fn assert_appearance_successor_cost(
    cost: worth_ui_host_contract::UiMountedPresentationProductionCost,
) {
    assert_eq!(cost.source_instances(), 1);
    assert_eq!(cost.commands_considered(), 0);
    assert_eq!(cost.command_index_lookups(), 0);
    assert_eq!(cost.order_lookups(), 0);
    assert_eq!(cost.retained_command_scans(), 0);
    assert_eq!(cost.retained_command_clones(), 0);
    assert_eq!(cost.projection_rows_materialized(), 0);
}

fn assert_zero_successor_cost(cost: worth_ui_host_contract::UiMountedPresentationProductionCost) {
    assert_eq!(cost.source_instances(), 0);
    assert_eq!(cost.commands_considered(), 0);
    assert_eq!(cost.command_index_lookups(), 0);
    assert_eq!(cost.order_lookups(), 0);
    assert_eq!(cost.retained_command_scans(), 0);
    assert_eq!(cost.retained_command_clones(), 0);
    assert_eq!(cost.projection_rows_materialized(), 0);
}

#[test]
#[ignore = "closure courtroom: compiles and mounts the full 2,048-surface public world"]
fn maximum_overlap_removals_cross_public_runtime_and_headless_with_exact_work() {
    let recorder = worth_ui_host_headless::WorthUiHeadlessRecorder::with_viewport_extent(
        worth_ui_host_headless::UiHeadlessRecorderCapacity::new(1, 2, 16_384),
        worth_ui::facade::measurement_exchange::UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let production = produce_maximum_overlap(recorder);
    let mut world = MountedPresentationWorld::maximum_overlap(
        &production.initial,
        production.authored_instances,
        production.semantic_surface,
    );
    assert_eq!(world.identity(), "mounted-presentation-world");
    assert_eq!(world.version(), 1);
    assert_eq!(world.baseline().len(), 2_048);
    assert_eq!(
        ordered_pixel(world.baseline(), [80, 48]),
        world.baseline().last().unwrap().rgba
    );
    assert_eq!(production.deltas.len(), 3);
    assert_eq!(production.restorations.len(), 2);
    world.assert_unchanged(&production.unchanged);
    world.assert_removal_delta(&production.deltas[0]);
    world.assert_restoration(&production.restorations[0]);
    world.assert_removal_delta(&production.deltas[1]);
    world.assert_restoration(&production.restorations[1]);
    world.assert_removal_delta(&production.deltas[2]);
    assert_required_oracle_mutations_are_rejected();
    let _ = production.session.shutdown();
}

#[test]
fn independent_oracle_rejects_each_required_control_mutation_for_its_exact_cause() {
    assert_required_oracle_mutations_are_rejected();
}

fn assert_required_oracle_mutations_are_rejected() {
    let baseline = [oracle::OracleRect {
        identity: 1,
        bounds: [4, 4, 8, 8],
        rgba: [1, 2, 3, 255],
        order: 1,
    }];
    let delta = oracle::OracleDelta {
        changes: vec![oracle::OracleRectChange {
            identity: 1,
            previous: baseline[0],
            successor: oracle::OracleRect {
                rgba: [9, 8, 7, 255],
                ..baseline[0]
            },
        }],
    };
    let expected = expectation(&baseline, &delta);
    let mut mutant = expected.clone();
    mutant.owner_delta_count = 0;
    assert_eq!(
        adjudicate(&expected, &mutant),
        Err(OracleDenial::OwnerDeltaDropped)
    );
    mutant = expected.clone();
    mutant.damage[0] = [0, 0, 160, 96];
    assert_eq!(
        adjudicate(&expected, &mutant),
        Err(OracleDenial::DamageWidened)
    );
    mutant = expected.clone();
    mutant.ordered_identities.clear();
    assert_eq!(
        adjudicate(&expected, &mutant),
        Err(OracleDenial::PaintOrderChanged)
    );
    mutant = expected.clone();
    mutant.vacated_damage_count = 0;
    assert_eq!(
        adjudicate(&expected, &mutant),
        Err(OracleDenial::VacatedDamageOmitted)
    );
}
