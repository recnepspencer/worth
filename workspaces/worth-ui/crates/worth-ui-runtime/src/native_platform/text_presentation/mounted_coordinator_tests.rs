use std::{collections::HashSet, sync::Arc};

use crate::certification_support::{
    initial_presentation_mechanics_for_certification, semantic_text_projection_for_certification,
    UiSemanticTextProjectionCertificationMutation,
};
use crate::mounting::presentation::coordinator::UiMountedTextPinState;
use crate::native_platform::text_presentation::UiNativeMountedTextCoordinator;
use worth_ui_host_contract::UiMountedPresentationWorkView;

#[path = "mounted_coordinator_test_harness.rs"]
mod harness;
use harness::{
    color_mechanic, glyph_geometry, initial_text, prepare, present, replacement_delta, text_command,
};

#[test]
fn qualified_text_coordinator_reuses_paint_only_work_and_denies_layout_twin() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = harness::requirement(&projection);
    let initial = initial_presentation_mechanics_for_certification(&projection, requirement);
    let layout = crate::mounting::qualified_text_test_support::inert_qualified_layout("ONLINE");
    let mut coordinator = UiNativeMountedTextCoordinator::default();
    let initial_prepared = prepare(
        &coordinator,
        UiMountedPresentationWorkView::Initial(&initial),
        requirement,
        layout.as_ref(),
    );
    let initial_demand = initial_prepared.demand_batches()[0].clone();
    let initial_geometry = glyph_geometry(&initial_prepared);
    assert!(!initial_prepared.foreground_reused());
    assert!(initial_prepared
        .performed_layout_work()
        .iter()
        .any(|value| *value != 0));
    assert!(!initial_demand.records().is_empty());
    let initial_candidate = coordinator
        .pins
        .candidate(requirement.binding(), &initial_prepared);
    let initial_pins = UiMountedTextPinState::binding_pins(&initial_candidate)
        .iter()
        .copied()
        .collect::<HashSet<_>>();

    let initial_observation = present(
        &mut coordinator,
        UiMountedPresentationWorkView::Initial(&initial),
        requirement,
        &initial_prepared,
        layout.as_ref(),
    );
    let (_, pending, _, _, reuse) = initial_observation.into_parts();
    assert!(pending.is_none());
    coordinator.commit_foreground_reuse(reuse.expect("presented text retains a reuse receipt"));
    let (observations, complete) = coordinator.take_work_observations();
    assert!(complete);
    assert_eq!(observations.len(), 1);
    let initial_counts = observations[0].work_counts();
    assert!(
        initial_counts[6] > 0,
        "initial presentation rasterizes misses"
    );
    assert!(
        initial_counts[9] > 0,
        "initial presentation adds atlas pins"
    );
    assert!(initial_counts[13..].iter().any(|value| *value != 0));

    let color_frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let color = text_command(
        initial_text(&initial),
        color_frame,
        Arc::from("ONLINE"),
        layout.as_ref(),
        [247, 129, 47, 255],
        true,
    );
    let color_delta = replacement_delta(initial.affinity(), color);
    let color_prepared = prepare(
        &coordinator,
        UiMountedPresentationWorkView::Delta(&color_delta),
        requirement,
        layout.as_ref(),
    );
    assert!(color_prepared.foreground_reused());
    assert_eq!(color_prepared.performed_layout_work(), [0; 17]);
    assert_eq!(color_prepared.demand_batches()[0], initial_demand);
    assert_eq!(glyph_geometry(&color_prepared), initial_geometry);
    assert_ne!(
        color_prepared.glyph_runs()[0].foreground(),
        initial_prepared.glyph_runs()[0].foreground()
    );
    let color_candidate = coordinator
        .pins
        .candidate(requirement.binding(), &color_prepared);
    assert!(color_candidate.has_no_pin_churn());
    let color_pins = UiMountedTextPinState::binding_pins(&color_candidate)
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    assert_eq!(color_pins, initial_pins);

    let color_observation = present(
        &mut coordinator,
        UiMountedPresentationWorkView::Delta(&color_delta),
        requirement,
        &color_prepared,
        layout.as_ref(),
    );
    let (_, pending, _, _, reuse) = color_observation.into_parts();
    assert!(pending.is_none());
    coordinator.commit_foreground_reuse(reuse.expect("color repaint retains a reuse receipt"));
    let (observations, complete) = coordinator.take_work_observations();
    assert!(complete);
    let color_counts = observations[0].work_counts();
    assert_eq!(color_counts[4], 0, "reuse skips demand record planning");
    assert_eq!(color_counts[6], 0, "reuse selects no raster misses");
    assert_eq!(color_counts[9], 0, "reuse adds no atlas pins");
    assert_eq!(color_counts[10], 0, "reuse releases no atlas pins");
    assert!(color_counts[13..].iter().all(|value| *value == 0));

    let changed_text = Arc::from("ONLINE longer");
    let changed_layout =
        crate::mounting::qualified_text_test_support::inert_qualified_layout(&changed_text);
    let layout_command = text_command(
        color_mechanic(&color_delta),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        changed_text,
        changed_layout.as_ref(),
        [247, 129, 47, 255],
        false,
    );
    let layout_delta = replacement_delta(color_delta.affinity(), layout_command);
    let layout_prepared = prepare(
        &coordinator,
        UiMountedPresentationWorkView::Delta(&layout_delta),
        requirement,
        changed_layout.as_ref(),
    );
    assert!(!layout_prepared.foreground_reused());
    assert!(layout_prepared
        .performed_layout_work()
        .iter()
        .any(|value| *value != 0));
    assert_ne!(
        layout_prepared.demand_batches()[0].layout_identity(),
        initial_demand.layout_identity()
    );
}
