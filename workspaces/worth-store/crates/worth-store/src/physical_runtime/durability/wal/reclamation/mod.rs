mod authority;
mod eligibility;
mod execution;
mod inventory_transition;
mod outcome;
mod work_port;

use authority::{EligiblePhysicalWalReclamation, EligiblePhysicalWalSegmentReclamation};
pub(super) use eligibility::{
    plan_reclamation, reclaimable_closed_prefix, PhysicalWalReclamationEligibilityDenial,
    PhysicalWalReclamationPlan,
};
pub(in crate::physical_runtime) use execution::{
    PhysicalWalReclamationFoundation, PhysicalWalReclamationOwner,
};
pub use outcome::{PhysicalWalReclamationObservation, PhysicalWalReclamationReport};
use work_port::{PhysicalWalReclamationActionFailure, PhysicalWalReclamationWorkPort};

#[cfg(test)]
mod tests;
