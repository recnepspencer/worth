mod admission;
mod profile;
mod retirement;

pub(in crate::physical_runtime) use admission::{
    CandidateGrowthLease, DisplacedSegment, GarbageClaim, PendingPublicationLease,
    PhysicalPublicationAdmission, PhysicalPublicationAdmissionDenial,
};
pub(in crate::physical_runtime) use profile::PhysicalRetentionProfile;
pub use retirement::PhysicalRetirementDenial;
pub(in crate::physical_runtime::durability) use retirement::note_retirement_hold;
pub(in crate::physical_runtime) use retirement::{
    decode_retirement, encode_retirement, payload_is_retirement, unresolved_retirement_holds,
    unresolved_retirements,
    RetirementRecord, RetirementRemovalPermit, RETIREMENT_COMPLETION, RETIREMENT_INTENT,
};