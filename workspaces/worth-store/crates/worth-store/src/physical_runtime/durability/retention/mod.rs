mod admission;
mod profile;
mod retired_artifact;
mod retirement;

pub(in crate::physical_runtime) use admission::{
    CandidateGrowthLease, DisplacedArtifact, GarbageClaim, PendingPublicationLease,
    PhysicalPublicationAdmission, PhysicalPublicationAdmissionDenial,
};
pub(in crate::physical_runtime) use profile::PhysicalRetentionProfile;
pub(in crate::physical_runtime) use retired_artifact::RetiredArtifact;
pub(in crate::physical_runtime::durability) use retirement::note_retirement_hold;
pub use retirement::PhysicalRetirementDenial;
pub(in crate::physical_runtime) use retirement::{
    decode_retirement, encode_retirement, payload_is_retirement, unresolved_retirement_holds,
    unresolved_retirements, RetirementRecord, RetirementRemovalPermit,
};
