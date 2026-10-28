use worth_server::{
    WorthServerBinaryCounterSet, WorthServerCompatibilityDenial,
    WorthServerCompatibilityDenialCode, WorthServerExternalCounterSet,
    WorthServerQueryHandoffDenial,
};

use crate::shared_support_compat_http_phase_thirteen_bundle::WorthServerPhaseThirteenBundle;

pub(crate) fn assert_bundle_digests_equal(
    left: &WorthServerPhaseThirteenBundle,
    right: &WorthServerPhaseThirteenBundle,
    labels: &[&'static str],
) {
    for label in labels {
        assert_eq!(
            left.digest(label),
            right.digest(label),
            "expected digest `{label}` to stay equal"
        );
    }
}

pub(crate) fn assert_bundle_digests_not_equal(
    left: &WorthServerPhaseThirteenBundle,
    right: &WorthServerPhaseThirteenBundle,
    labels: &[&'static str],
) {
    for label in labels {
        assert_ne!(
            left.digest(label),
            right.digest(label),
            "expected digest `{label}` to stay distinct"
        );
    }
}

pub(crate) fn assert_external_counter(
    counters: &WorthServerExternalCounterSet,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        counters.counter(name),
        Some(expected),
        "expected external counter `{name}` to equal `{expected}`",
    );
}

pub(crate) fn assert_binary_counter(
    counters: &WorthServerBinaryCounterSet,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        counters.counter(name),
        Some(expected),
        "expected binary counter `{name}` to equal `{expected}`",
    );
}

pub(crate) fn assert_external_counters_zero(
    counters: &WorthServerExternalCounterSet,
    names: &[&str],
) {
    for name in names {
        assert_external_counter(counters, name, 0);
    }
}

pub(crate) fn assert_denial_contains(
    denial: &WorthServerQueryHandoffDenial,
    expected_code: worth_server::WorthServerQueryHandoffDenialCode,
    expected_detail_fragment: &str,
) {
    assert_eq!(denial.code(), expected_code);
    assert!(
        denial.detail().contains(expected_detail_fragment),
        "expected denial detail to contain `{expected_detail_fragment}`, got `{}`",
        denial.detail()
    );
}

pub(crate) fn assert_compatibility_denial_contains(
    denial: &WorthServerCompatibilityDenial,
    expected_code: WorthServerCompatibilityDenialCode,
    expected_detail_fragment: &str,
) {
    assert_eq!(denial.code(), expected_code);
    assert!(
        denial.detail().contains(expected_detail_fragment),
        "expected denial detail to contain `{expected_detail_fragment}`, got `{}`",
        denial.detail()
    );
}
