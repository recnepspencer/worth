use crate::external_observation::{NativeClientAreaBounds, NativeClientPixelCapture};

pub(super) struct NativePhase6WorldFacts<'world> {
    pub(super) capture: &'world NativeClientPixelCapture,
    pub(super) bounds: NativeClientAreaBounds,
    pub(super) click_screen: (i32, i32),
    pub(super) moved_screen: (i32, i32),
}

pub(super) fn assert_phase6_evidence(
    evidence: &serde_json::Value,
    facts: NativePhase6WorldFacts<'_>,
) {
    assert_presentation_identity(evidence, facts.capture);
    let input = &evidence["input"];
    assert_runtime_settlement(evidence, input);
    assert_terminal_cleanup(evidence);
    assert_scroll_continuity(input);
    assert_event_time_pointer_affinity(evidence, input, facts);
}

fn assert_presentation_identity(evidence: &serde_json::Value, capture: &NativeClientPixelCapture) {
    assert_eq!(evidence["schema"], "worth-ui-native-phase6-evidence-v1");
    assert_eq!(
        evidence["presentation"]["client_physical_size"],
        serde_json::json!([capture.width(), capture.height()])
    );
}

fn assert_runtime_settlement(evidence: &serde_json::Value, input: &serde_json::Value) {
    assert!(input["retained_events"]
        .as_u64()
        .is_some_and(|count| count > 0));
    let retained_batches = input["retained_batches"]
        .as_u64()
        .expect("the retained report includes a batch count");
    assert!(retained_batches > 0);
    let ingress = evidence["runtime_ingress"]
        .as_object()
        .expect("phase 6 evidence includes runtime ingress settlement");
    assert!(ingress["applied_batches"]
        .as_u64()
        .is_some_and(|count| count > 0));
    assert_eq!(ingress["drain_denied"], 0);
    assert!(ingress["typed_disposition_count"]
        .as_u64()
        .is_some_and(|count| count >= retained_batches));
}

fn assert_terminal_cleanup(evidence: &serde_json::Value) {
    assert_eq!(evidence["terminal_zero"], true);
    let census = evidence["terminal_census"]
        .as_object()
        .expect("phase 6 evidence includes terminal census");
    assert!(!census.is_empty());
    assert!(census.values().all(|value| value.as_u64() == Some(0)));
}

fn assert_scroll_continuity(input: &serde_json::Value) {
    let vertical = input["last_vertical_scroll"]
        .as_object()
        .expect("the real Windows vertical wheel reaches host-native translation");
    let horizontal = input["last_horizontal_scroll"]
        .as_object()
        .expect("the real Windows horizontal wheel reaches host-native translation");
    // One notch carries the platform's own line count, in thousandths of a line,
    // read here from the same documented setting store by the courtroom's own hand.
    let lines = i64::from(
        crate::adjudication::platform_wheel_lines_per_notch()
            .expect("the platform wheel-lines setting is a count or absent"),
    );
    assert_eq!(vertical["x_subpixels"], 0);
    assert_eq!(vertical["y_subpixels"], lines * 1_000);
    assert_eq!(vertical["lines_per_notch"], lines);
    assert_eq!(horizontal["x_subpixels"], -lines * 1_000);
    assert_eq!(horizontal["y_subpixels"], 0);
    assert_eq!(horizontal["lines_per_notch"], lines);
    for basis in [
        &vertical["line_count_basis"],
        &horizontal["line_count_basis"],
    ] {
        assert!(
            basis == "PlatformReported" || basis == "DefaultedAfterMissing",
            "the host relays the platform count or the documented default: {basis}"
        );
    }
    let vertical_sequence = vertical["sequence"]
        .as_u64()
        .expect("vertical scroll has a retained sequence");
    let horizontal_sequence = horizontal["sequence"]
        .as_u64()
        .expect("horizontal scroll has a retained sequence");
    assert!(vertical_sequence < horizontal_sequence);
    assert!(input["last_sequence"]
        .as_u64()
        .is_some_and(|last| last > horizontal_sequence));
}

fn assert_event_time_pointer_affinity(
    evidence: &serde_json::Value,
    input: &serde_json::Value,
    facts: NativePhase6WorldFacts<'_>,
) {
    let button = input["last_pointer_button"]
        .as_object()
        .expect("the retained report includes the pointer button witness");
    let click_client = client_point(facts.click_screen, facts.bounds);
    let moved_client = client_point(facts.moved_screen, facts.bounds);
    let scale_factor_milli = evidence["presentation"]["scale_factor_milli"]
        .as_i64()
        .expect("the presentation reports its observed scale factor");
    assert!(scale_factor_milli > 0);
    let click_surface = logical_point(click_client, scale_factor_milli);
    let moved_surface = logical_point(moved_client, scale_factor_milli);
    let observed = (
        button["x_subpixels"]
            .as_i64()
            .expect("pointer x is an integer subpixel coordinate"),
        button["y_subpixels"]
            .as_i64()
            .expect("pointer y is an integer subpixel coordinate"),
    );
    assert_pointer_coordinates(
        observed,
        click_surface,
        moved_surface,
        scale_factor_milli,
        facts.bounds,
    );
    assert_eq!(button["coordinate_space"], "Viewport");
    assert_eq!(button["coordinate_unit"], "LogicalPoint");
    assert!(input["last_sequence"].as_u64().is_some_and(|sequence| {
        button["sequence"]
            .as_u64()
            .is_some_and(|button_sequence| sequence >= button_sequence && button_sequence > 0)
    }));
}

fn assert_pointer_coordinates(
    observed: (i64, i64),
    click_surface: (i64, i64),
    moved_surface: (i64, i64),
    scale_factor_milli: i64,
    bounds: NativeClientAreaBounds,
) {
    assert!(
        within_subpixels(observed.0, click_surface.0, 3_000),
        "event-time x mismatch: observed={observed:?}; click_surface={click_surface:?}; moved_surface={moved_surface:?}; scale_factor_milli={scale_factor_milli}; bounds={bounds:?}",
    );
    assert!(
        within_subpixels(observed.1, click_surface.1, 3_000),
        "event-time y mismatch: observed={observed:?}; click_surface={click_surface:?}; moved_surface={moved_surface:?}; scale_factor_milli={scale_factor_milli}; bounds={bounds:?}",
    );
    assert!(
        !within_subpixels(observed.0, moved_surface.0, 3_000)
            || !within_subpixels(observed.1, moved_surface.1, 3_000),
        "the retained pointer position must not be reconstructed from the post-delivery cursor"
    );
}

fn client_point(screen: (i32, i32), bounds: NativeClientAreaBounds) -> (i64, i64) {
    (
        i64::from(screen.0 - bounds.left()),
        i64::from(screen.1 - bounds.top()),
    )
}

fn logical_point(point: (i64, i64), scale_factor_milli: i64) -> (i64, i64) {
    (
        logical_subpixels(point.0, scale_factor_milli),
        logical_subpixels(point.1, scale_factor_milli),
    )
}

fn within_subpixels(actual: i64, expected: i64, tolerance: i64) -> bool {
    actual.abs_diff(expected) <= tolerance as u64
}

fn logical_subpixels(physical_coordinate: i64, scale_factor_milli: i64) -> i64 {
    (physical_coordinate * 1_000_000 + scale_factor_milli / 2) / scale_factor_milli
}
