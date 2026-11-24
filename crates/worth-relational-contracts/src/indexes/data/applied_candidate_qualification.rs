//! Descriptive applied index-read identity, independent of the live pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateReadPhase {
    AssemblyRevision(u64),
    PreparedCandidate(u64),
}

/// Compact descriptive qualification for values read from one applied image.
/// It retains no proposal, source context, record payload or runtime handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalAppliedCandidateReadQualification {
    pub(crate) runtime_instance_id: u64,
    pub(crate) proposal_ordinal: u64,
    pub(crate) proposed_version: crate::identity::data::VersionId,
    pub(crate) phase: CandidateReadPhase,
}
impl RelationalAppliedCandidateReadQualification {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub const fn proposed_version_id(&self) -> crate::identity::data::VersionId {
        self.proposed_version
    }
}

pub const fn assemble_applied_candidate_read_qualification(
    runtime_instance_id: u64,
    proposal_ordinal: u64,
    proposed_version: crate::identity::data::VersionId,
    phase: CandidateReadPhase,
) -> RelationalAppliedCandidateReadQualification {
    RelationalAppliedCandidateReadQualification {
        runtime_instance_id,
        proposal_ordinal,
        proposed_version,
        phase,
    }
}
