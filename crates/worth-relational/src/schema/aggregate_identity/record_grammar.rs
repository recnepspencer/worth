//! Reserved aggregate and lineage record field declarations.
use worth_foundational::facade::ScalarAspectType;

pub(crate) const AGGREGATE_ASPECT: &str = "aggregate-identity";
pub(crate) const LINEAGE_ASPECT: &str = "aggregate-lineage";
pub(crate) const AGGREGATE_FIELDS: &[(&str, ScalarAspectType)] = &[
    ("identity-high", ScalarAspectType::Uuid),
    ("identity-low", ScalarAspectType::Uuid),
    ("definition-high", ScalarAspectType::Uuid),
    ("definition-low", ScalarAspectType::Uuid),
    ("born", ScalarAspectType::UInt64),
    ("seed-partition", ScalarAspectType::UInt32),
    ("seed-slot", ScalarAspectType::UInt64),
    ("seed-generation", ScalarAspectType::UInt32),
    ("current-partition", ScalarAspectType::UInt32),
    ("current-slot", ScalarAspectType::UInt64),
    ("current-generation", ScalarAspectType::UInt32),
    ("active", ScalarAspectType::Bool),
    ("retired", ScalarAspectType::UInt64),
];
pub(crate) const LINEAGE_FIELDS: &[(&str, ScalarAspectType)] = &[
    ("occurrence", ScalarAspectType::UInt64),
    ("kind", ScalarAspectType::UInt8),
];
