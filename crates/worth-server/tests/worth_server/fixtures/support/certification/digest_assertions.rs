#![allow(dead_code)]

pub(crate) use super::certification_bundle::{
    WorthServerCertificationBundle, WorthServerCertificationField,
};

pub(crate) fn assert_equal_on(
    left: &WorthServerCertificationBundle,
    right: &WorthServerCertificationBundle,
    fields: &[WorthServerCertificationField],
) {
    for field in fields {
        match comparable_field(left, right, field) {
            ComparableField::String(left, right) => {
                assert_eq!(left, right, "field {}", field.label())
            }
            ComparableField::OptionalString(left, right) => {
                assert_eq!(left, right, "field {}", field.label())
            }
            ComparableField::CounterSnapshot(left, right) => {
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
            ComparableField::String(left, right) => {
                assert_ne!(left, right, "field {}", field.label())
            }
            ComparableField::OptionalString(left, right) => {
                assert_ne!(left, right, "field {}", field.label())
            }
            ComparableField::CounterSnapshot(left, right) => {
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
        WorthServerCertificationField::RequestContextDigest => ComparableField::String(
            left.request_context_digest(),
            right.request_context_digest(),
        ),
        WorthServerCertificationField::ResponseDigest => {
            ComparableField::String(left.response_digest(), right.response_digest())
        }
        WorthServerCertificationField::ProvenanceDigest => {
            ComparableField::String(left.provenance_digest(), right.provenance_digest())
        }
        WorthServerCertificationField::FailureDigest => {
            ComparableField::OptionalString(left.failure_digest(), right.failure_digest())
        }
        WorthServerCertificationField::CounterSnapshot => {
            ComparableField::CounterSnapshot(left.counter_snapshot(), right.counter_snapshot())
        }
        WorthServerCertificationField::Output(output) => {
            let left_output = left.output_digest(*output);
            let right_output = right.output_digest(*output);
            ComparableField::OptionalString(left_output, right_output)
        }
    }
}
