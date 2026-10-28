//! The additional private grammar used only by projected aggregate bindings.
mod binding;
mod case_roster;
mod domain_text;

pub(super) use binding::{domain_for_contract, member_key};
pub(super) use case_roster::{decode_cases, encode_cases};
pub(super) use domain_text::{decode_domain, encode_domain};
use worth_foundational::facade::ScalarAspectType;

pub(crate) const COORDINATE_ASPECT: &str = "aggregate-projected-coordinate";
pub(crate) const MEMBERSHIP_ASPECT: &str = "aggregate-projected-membership";
pub(crate) const COORDINATE_FIELDS: &[(&str, ScalarAspectType)] = &[
    ("domain", ScalarAspectType::String),
    ("seed-case", ScalarAspectType::String),
    ("current-case", ScalarAspectType::String),
];
pub(crate) const MEMBERSHIP_FIELDS: &[(&str, ScalarAspectType)] =
    &[("cases", ScalarAspectType::String)];
