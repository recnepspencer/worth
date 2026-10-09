//! # STATE GRAPH
//!
//! Movement planning law only. A completed lower read plan is not Store byte I/O.
//!
//! - **Read-plan bookkeeping** enters through [`BlobPlacementMovementReadPlanBasis`]
//!   (physical-isolation plan completion plus migration interlock).
//! - **Security-scope evidence** enters through [`BlobPlacementMovementForegroundReservation`]
//!   admitted scope identity matched against the lifecycle declaration.
//! - **Cold-tier posture evidence** enters through [`BlobPlacementMovementColdOutcome`], classified
//!   via tiering [`cold_posture_permits_movement`] before movement planning admits.
//!
//! This module does not execute, publish, or certify a physical movement. Store
//! owns the future physical tier-movement authority and byte-read proof.
//!
//! Primary entry points:
//! - [`BlobPlacementMovementAuthority::plan_movement`] — classify eligibility via composed
//!   `require_*` verification steps, then construct movement plan.

mod classification;
mod counters;
mod denial;
mod orchestration;
mod receipt_construction;
mod transitions;
mod types;
mod verification;

#[cfg(test)]
mod tests;

pub use counters::BlobPlacementMovementCounterSnapshot;
pub use denial::BlobPlacementMovementDenial;
pub use types::{
    AdmittedBlobPlacementMovementPlan, BlobPlacementMovementAuthority,
    BlobPlacementMovementColdCapsuleOutcome, BlobPlacementMovementColdExportOutcome,
    BlobPlacementMovementColdMaterializationOutcome, BlobPlacementMovementColdOutcome,
    BlobPlacementMovementColdReadOutcome, BlobPlacementMovementForegroundReservation,
    BlobPlacementMovementFreshness, BlobPlacementMovementReadPlanBasis,
    BlobPlacementMovementRequest,
};
