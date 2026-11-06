mod admission;
mod canonical_map;
mod record_state;

pub use admission::{admit_authoritative_record_aspect_state, AuthoritativeStateAdmissionDenial};
pub use canonical_map::CanonicalAspectStateMap;
pub use record_state::{
    AuthoritativeRecordAspectState, AuthoritativeRecordAspectStateAdmitted,
    AuthoritativeRecordAspectStateArtifact,
};
