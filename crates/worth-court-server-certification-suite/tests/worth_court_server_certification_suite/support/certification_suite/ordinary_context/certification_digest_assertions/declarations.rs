#![allow(dead_code)]

pub(crate) use {crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField};

pub(crate) fn assert_equal_on(
    left: &WorthServerCertificationBundle,
    right: &WorthServerCertificationBundle,
    fields: &[WorthServerCertificationField],
) {
    for field in fields {
        match comparable_field(left, right, field) {
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::String(left, right) => {
                assert_eq!(left, right, "field {}", field.label())
            }
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::OptionalString(left, right) => {
                assert_eq!(left, right, "field {}", field.label())
            }
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::CounterSnapshot(left, right) => {
                assert_eq!(left, right, "field {}", field.label())
            }
        }
    }
}

pub(crate) fn assert_not_equal_on(
    left: &WorthServerCertificationBundle,
    right: &WorthServerCertificationBundle,
    fields: &[WorthServerCertificationField],
) {
    for field in fields {
        match comparable_field(left, right, field) {
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::String(left, right) => {
                assert_ne!(left, right, "field {}", field.label())
            }
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::OptionalString(left, right) => {
                assert_ne!(left, right, "field {}", field.label())
            }
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::CounterSnapshot(left, right) => {
                assert_ne!(left, right, "field {}", field.label())
            }
        }
    }
}

pub(crate) enum ComparableField<'a> {
    String(&'a str, &'a str),
    OptionalString(Option<&'a str>, Option<&'a str>),
    CounterSnapshot(
        &'a std::collections::BTreeMap<String, u64>,
        &'a std::collections::BTreeMap<String, u64>,
    ),
}

pub(crate) fn comparable_field<'a>(
    left: &'a WorthServerCertificationBundle,
    right: &'a WorthServerCertificationBundle,
    field: &WorthServerCertificationField,
) -> ComparableField<'a> {
    match field {
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest => crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::String(
            left.request_context_digest(),
            right.request_context_digest(),
        ),
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest => {
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::String(left.response_digest(), right.response_digest())
        }
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ProvenanceDigest => {
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::String(left.provenance_digest(), right.provenance_digest())
        }
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest => {
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::OptionalString(left.failure_digest(), right.failure_digest())
        }
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::CounterSnapshot => {
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::CounterSnapshot(left.counter_snapshot(), right.counter_snapshot())
        }
        crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(output) => {
            let left_output = left.output_digest(*output);
            let right_output = right.output_digest(*output);
            crate::certification_suite::ordinary_context::certification_digest_assertions::ComparableField::OptionalString(left_output, right_output)
        }
    }
}