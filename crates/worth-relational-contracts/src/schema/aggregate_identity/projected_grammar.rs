//! Reserved fields for projected aggregate and membership coordinates.
use worth_foundational::facade::ScalarAspectType;

pub const COORDINATE_ASPECT: &str = "aggregate-projected-coordinate";
pub const MEMBERSHIP_ASPECT: &str = "aggregate-projected-membership";
pub const COORDINATE_FIELDS: &[(&str, ScalarAspectType)] = &[
    ("domain", ScalarAspectType::String),
    ("seed-case", ScalarAspectType::String),
    ("current-case", ScalarAspectType::String),
];
pub const MEMBERSHIP_FIELDS: &[(&str, ScalarAspectType)] = &[("cases", ScalarAspectType::String)];
