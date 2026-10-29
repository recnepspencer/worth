#![allow(dead_code)]

pub(crate) use {crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle};

pub(crate) fn assert_counter_exact(
    bundle: &WorthServerCertificationBundle,
    counter_name: &str,
    expected_exact_value: u64,
) {
    assert_eq!(
        bundle
            .counter_value(counter_name)
            .unwrap_or_else(|| panic!("missing counter {counter_name}")),
        expected_exact_value,
        "unexpected exact counter value for {counter_name}"
    );
}

pub(crate) fn assert_counters_zero(
    bundle: &WorthServerCertificationBundle,
    counter_names: &[&str],
) {
    for counter_name in counter_names {
        assert_counter_exact(bundle, counter_name, 0);
    }
}