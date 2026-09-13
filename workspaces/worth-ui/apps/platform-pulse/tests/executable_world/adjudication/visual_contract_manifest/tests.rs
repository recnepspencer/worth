use super::{checked_in, validation, SOURCE};

#[test]
fn checked_in_visual_contract_is_independent_complete_and_bounded() {
    let manifest = checked_in().expect("the 3.15 visual contract is complete");
    assert!(manifest
        .layouts
        .iter()
        .any(|layout| layout.name == "default" && layout.logical_client_extent == [960, 600]));
    assert!(manifest.layouts.iter().any(|layout| {
        layout.name == "resized" && layout.logical_client_extent == [1_120, 700]
    }));
    assert!(manifest
        .layouts
        .iter()
        .all(|layout| layout.minimum_targets.len() == 3));
    assert!(manifest.layouts.iter().all(|layout| {
        let portal = layout
            .minimum_targets
            .iter()
            .find(|target| target.identity == "platform.pulse.target.open_portal");
        portal.is_some_and(|target| {
            target.action_identity == "intent:platform.pulse.portal.open.route:activate"
                && target.label_identity == "platform.pulse.text.portal_label"
                && layout
                    .text_bounds
                    .iter()
                    .any(|text| text.identity == target.label_identity)
        })
    }));
    assert_eq!(manifest.limits.maximum_capture_rgba_bytes, 50_176_000);
    assert_eq!(
        manifest.limits.maximum_retained_capture_rgba_bytes,
        100_352_000,
    );
}

#[test]
fn visual_contract_rejects_decorative_hit_testing() {
    let mut manifest =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    manifest.layouts[0].regions[0].hit_test = true;
    assert_eq!(
        validation::validate(&manifest),
        Err(validation::PlatformPulseVisualContractFailure::Geometry),
    );
}

#[test]
fn visual_contract_rejects_fake_controls_and_authored_palette_drift() {
    let mut fake =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    fake.layouts[0].minimum_targets[0].visibility_gate = "always-visible".into();
    assert_eq!(
        validation::validate(&fake),
        Err(validation::PlatformPulseVisualContractFailure::Target),
    );

    let mut low_contrast =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    let secondary = low_contrast
        .tokens
        .iter_mut()
        .find(|token| token.role == "secondary-text")
        .unwrap();
    secondary.rgba = [250, 250, 250, 255];
    assert_eq!(
        validation::validate(&low_contrast),
        Err(validation::PlatformPulseVisualContractFailure::Token),
    );
}

#[test]
fn primary_action_text_clears_body_contrast_in_both_themes() {
    assert!(super::contrast::ratio_milli([250, 250, 250, 255], [255, 255, 255, 255]) < 4_500);
    for foreground in [[242, 244, 247, 255], [250, 251, 252, 255]] {
        for background in [[148, 64, 212, 255], [160, 84, 24, 255]] {
            assert!(super::contrast::ratio_milli(foreground, background) >= 4_500);
        }
        assert!(super::contrast::ratio_milli(foreground, [172, 103, 242, 255]) < 4_500);
    }
}

#[test]
fn structural_rules_clear_the_non_text_contrast_floor_on_every_surface() {
    let manifest = checked_in().expect("the checked-in visual contract validates");
    let structural = manifest
        .tokens
        .iter()
        .find(|token| token.role == "structural-rule")
        .expect("structural rule token")
        .rgba;
    let structural_pairs = manifest
        .contrast_pairs
        .iter()
        .filter(|pair| pair.foreground == "structural-rule")
        .collect::<Vec<_>>();
    assert_eq!(structural_pairs.len(), 3);
    for pair in structural_pairs {
        let background = manifest
            .tokens
            .iter()
            .find(|token| token.role == pair.background)
            .expect("structural background token")
            .rgba;
        assert_eq!(pair.minimum_ratio_milli, 3_000);
        assert!(super::contrast::ratio_milli(structural, background) >= 3_000);
    }
}

#[test]
fn visual_contract_rejects_an_unlabelled_or_retokened_control() {
    let mut unlabelled =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    unlabelled.layouts[0].minimum_targets[1].label_identity = "platform.pulse.text.missing".into();
    assert_eq!(
        validation::validate(&unlabelled),
        Err(validation::PlatformPulseVisualContractFailure::Target),
    );

    let mut retokened =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    retokened.layouts[0].minimum_targets[1].token_role = "canvas".into();
    assert_eq!(
        validation::validate(&retokened),
        Err(validation::PlatformPulseVisualContractFailure::Target),
    );
}

#[test]
fn visual_contract_rejects_text_escape_and_unbounded_capture_math() {
    let mut escaped =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    escaped.layouts[0].text_bounds[0].rect[0] = 900;
    assert_eq!(
        validation::validate(&escaped),
        Err(validation::PlatformPulseVisualContractFailure::TextContainment),
    );

    let mut unbounded =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    unbounded.limits.maximum_capture_scale = 5;
    assert_eq!(
        validation::validate(&unbounded),
        Err(validation::PlatformPulseVisualContractFailure::CaptureBudget),
    );
}

#[test]
fn visual_contract_rejects_wrapping_boxes_that_only_repeat_line_height() {
    let mut clipped =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    let source_posture = clipped.layouts[0]
        .text_bounds
        .iter_mut()
        .find(|text| text.identity == "platform.pulse.text.source_query")
        .unwrap();
    source_posture.rect[3] = 40;
    assert_eq!(
        validation::validate(&clipped),
        Err(validation::PlatformPulseVisualContractFailure::TextContainment),
    );

    let mut clipped =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    let query_posture = clipped.layouts[0]
        .text_bounds
        .iter_mut()
        .find(|text| text.identity == "platform.pulse.text.projected_status")
        .unwrap();
    query_posture.rect[3] = 40;
    assert_eq!(
        validation::validate(&clipped),
        Err(validation::PlatformPulseVisualContractFailure::TextContainment),
    );
}

#[test]
fn visual_contract_rejects_overflow_and_substituted_design_roles() {
    let mut overflow =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    overflow.layouts[0].text_bounds[0].rect = [u32::MAX, 40, 2, 24];
    assert_eq!(
        validation::validate(&overflow),
        Err(validation::PlatformPulseVisualContractFailure::TextContainment),
    );

    let mut substituted =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    substituted.typography[0].role = "decorative-display".into();
    assert_eq!(
        validation::validate(&substituted),
        Err(validation::PlatformPulseVisualContractFailure::Typography),
    );
}

#[test]
fn resized_text_and_targets_are_adjudicated_in_the_resized_regions() {
    let mut stale_resized =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    let resized = stale_resized
        .layouts
        .iter_mut()
        .find(|layout| layout.name == "resized")
        .unwrap();
    resized
        .text_bounds
        .iter_mut()
        .find(|text| text.identity == "platform.pulse.text.status")
        .unwrap()
        .rect = [40, 556, 880, 16];
    assert_eq!(
        validation::validate(&stale_resized),
        Err(validation::PlatformPulseVisualContractFailure::TextContainment),
    );

    let mut escaped_target =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    escaped_target
        .layouts
        .iter_mut()
        .find(|layout| layout.name == "resized")
        .unwrap()
        .minimum_targets[0]
        .rect = [1_100, 304, 192, 40];
    assert_eq!(
        validation::validate(&escaped_target),
        Err(validation::PlatformPulseVisualContractFailure::Target),
    );

    let mut escaped_control =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    escaped_control
        .layouts
        .iter_mut()
        .find(|layout| layout.name == "resized")
        .unwrap()
        .control_points[0]
        .logical_point = [1_120, 699];
    assert_eq!(
        validation::validate(&escaped_control),
        Err(validation::PlatformPulseVisualContractFailure::ControlPoint),
    );

    let mut misattributed_control =
        serde_json::from_str::<super::model::PlatformPulseVisualContractManifest>(SOURCE).unwrap();
    misattributed_control.layouts[0]
        .control_points
        .iter_mut()
        .find(|point| point.identity == "live-action")
        .unwrap()
        .logical_point = [8, 8];
    assert_eq!(
        validation::validate(&misattributed_control),
        Err(validation::PlatformPulseVisualContractFailure::ControlPoint),
    );
}
