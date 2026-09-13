use crate::validation::IndeterminatePhysicalIntegrityPosture;

use super::inspection::PhysicalIntegrityScrubInspection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityScrubWindowOutcome {
    Inspected(PhysicalIntegrityScrubInspection),
    Indeterminate(IndeterminatePhysicalIntegrityPosture),
}
