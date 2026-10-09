// --- Capabilities (admission handles, next-step types) ---
pub use crate::placement::{
    AdmittedBlobPlacement, AdmittedBlobPlacementMovementPlan, BlobPlacementAdmissionAuthority,
    BlobPlacementIntent, BlobPlacementMovementAuthority,
    BlobPlacementMovementForegroundReservation, BlobPlacementMovementReadPlanBasis,
    BlobPlacementMovementRequest, BlobPlacementNonClaim, BlobPlacementProof,
};
// --- Outcomes (transition receipts) ---
pub use crate::placement::{
    BlobPlacementMovementColdCapsuleOutcome, BlobPlacementMovementColdExportOutcome,
    BlobPlacementMovementColdMaterializationOutcome, BlobPlacementMovementColdOutcome,
    BlobPlacementMovementColdReadOutcome, BlobPlacementMovementFreshness,
};
// --- Denials (classified failure enums) ---
pub use crate::placement::{
    BlobPlacementAdmissionDenial, BlobPlacementClass, BlobPlacementMovementDenial,
};
// --- Counter witnesses (read-only snapshots) ---
pub use crate::placement::{BlobPlacementCounterSnapshot, BlobPlacementMovementCounterSnapshot};
