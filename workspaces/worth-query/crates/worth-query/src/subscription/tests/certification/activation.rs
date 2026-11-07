use super::activation_world::admitted_activation_for;
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn admitted_activation_emits_query_subscription_certification_bundle() {
    let (admission, activation, scale_report) = admitted_activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let admission_digest = admission.admission_projection().label().to_string();
    let activation_digest = activation.activation_projection().label().to_string();
    let scale_slope_digest = scale_report.report_projection().label().to_string();
    let bundle =
        certify_query_subscription_activation(admission, activation, scale_report).unwrap();

    assert!(!bundle.certification_bundle_projection().label().is_empty());
    assert_eq!(
        bundle.admission_projection().label(),
        admission_digest.as_str()
    );
    assert_eq!(
        bundle.activation_projection().label(),
        activation_digest.as_str()
    );
    assert_eq!(
        bundle.scale_slope_projection().label(),
        scale_slope_digest.as_str()
    );
    assert_eq!(
        bundle.scale_activation_projection().label(),
        activation_digest.as_str()
    );
    assert_eq!(
        bundle.scale_admission_projection().label(),
        admission_digest.as_str()
    );
    assert!(!bundle.support_profile_projection().label().is_empty());
    assert!(!bundle.diagnostics_projection().label().is_empty());
    assert!(!bundle.admission_counter_projection().label().is_empty());
    assert!(!bundle.activation_counter_projection().label().is_empty());
}

#[test]
fn certification_denies_activation_from_different_admission() {
    let (admission, _, scale_report) = admitted_activation_for(LiveQueryFamily::Detail, None);
    let (_, foreign_activation, _) = admitted_activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );

    let error = certify_query_subscription_activation(admission, foreign_activation, scale_report)
        .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionCertificationDenialKind::ActivationAdmissionMismatch
    );
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}

#[test]
fn certification_denies_scale_report_from_different_activation() {
    let (admission, activation, _) = admitted_activation_for(LiveQueryFamily::Detail, None);
    let (_, _, foreign_scale_report) = admitted_activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );

    let error = certify_query_subscription_activation(admission, activation, foreign_scale_report)
        .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionCertificationDenialKind::ScaleSlopeSourceMismatch
    );
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}

#[test]
fn scale_slope_certification_admits_row_count_only_variation() {
    let (_, activation, _) = admitted_activation_for(LiveQueryFamily::Detail, None);
    let report = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            1,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            10,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            100,
            &activation,
        )
        .expect("fixture digest must fit"),
    )
    .unwrap();

    assert_eq!(
        report.activation_projection().label(),
        activation.activation_projection().label().as_str()
    );
    assert_eq!(
        report.admission_projection().label(),
        activation.admission_projection().label().as_str()
    );
    assert_eq!(report.small_row_count(), 1);
    assert_eq!(report.medium_row_count(), 10);
    assert_eq!(report.large_row_count(), 100);
    assert!(!report.structural_counter_projection().label().is_empty());
}

#[test]
fn scale_slope_certification_denies_mixed_activation_sources() {
    let (_, activation, _) = admitted_activation_for(LiveQueryFamily::Detail, None);
    let (_, foreign_activation, _) = admitted_activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );

    let error = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            1,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            10,
            &foreign_activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            100,
            &activation,
        )
        .expect("fixture digest must fit"),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionCertificationDenialKind::ScaleSlopeDrift
    );
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}

#[test]
fn scale_slope_certification_denies_zero_row_baseline() {
    let (_, activation, _) = admitted_activation_for(LiveQueryFamily::Detail, None);

    let error = certify_query_subscription_scale_slope(
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Small,
            0,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Medium,
            10,
            &activation,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionScaleCounterSnapshot::from_activation(
            QuerySubscriptionScaleFixtureSize::Large,
            100,
            &activation,
        )
        .expect("fixture digest must fit"),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionCertificationDenialKind::ScaleSlopeDrift
    );
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}

#[test]
fn scale_slope_certification_denies_structural_counter_drift() {
    let (_, activation, _) = admitted_activation_for(LiveQueryFamily::Detail, None);
    let small = QuerySubscriptionScaleCounterSnapshot::from_activation(
        QuerySubscriptionScaleFixtureSize::Small,
        1,
        &activation,
    )
    .expect("fixture digest must fit");
    let medium = QuerySubscriptionScaleCounterSnapshot::from_activation(
        QuerySubscriptionScaleFixtureSize::Medium,
        10,
        &activation,
    )
    .expect("fixture digest must fit")
    .with_bridge_slice_count_for_test(&activation, activation.counters().bridge_slice_count() + 1);
    let large = QuerySubscriptionScaleCounterSnapshot::from_activation(
        QuerySubscriptionScaleFixtureSize::Large,
        100,
        &activation,
    )
    .expect("fixture digest must fit");

    let error = certify_query_subscription_scale_slope(small, medium, large).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionCertificationDenialKind::ScaleSlopeDrift
    );
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}
