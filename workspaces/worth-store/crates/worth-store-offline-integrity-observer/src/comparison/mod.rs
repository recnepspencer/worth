mod disagreement;
mod outcome;
mod protocol;
mod request;

pub use disagreement::compare_integrity_observations;
pub use outcome::{
    PhysicalIntegrityComparison, PhysicalIntegrityComparisonCounters,
    PhysicalIntegrityComparisonDenial,
};
pub use request::{PhysicalIntegrityComparisonLimits, PhysicalIntegrityComparisonLimitsDenial};
