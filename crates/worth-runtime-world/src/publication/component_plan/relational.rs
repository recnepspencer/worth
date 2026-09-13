use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;

/// Relational owner posture for one publication. It is separate from Signal
/// posture so a sibling cannot be silently refreshed or omitted. Branch
/// creation is not a publication posture and has its own plan vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalComponentPlanPosture {
    RetainExact,
    PublishPrepared,
}

#[derive(Debug)]
pub struct RelationalComponentPlan {
    posture: RelationalComponentPlanPosture,
    expected: AdmittedRelationalBranchBasis,
    prepared_candidate: Option<PreparedRelationalCommitCandidate>,
}

impl RelationalComponentPlan {
    pub const fn posture(&self) -> RelationalComponentPlanPosture {
        self.posture
    }

    pub const fn expected(&self) -> &AdmittedRelationalBranchBasis {
        &self.expected
    }

    /// The owner-issued candidate, when this plan carries ordinary
    /// Relational publication evidence. Borrowing it never transfers or
    /// duplicates the candidate's linear authority.
    pub fn prepared_candidate(&self) -> Option<&PreparedRelationalCommitCandidate> {
        self.prepared_candidate.as_ref()
    }

    pub(crate) fn retain_exact(expected: AdmittedRelationalBranchBasis) -> Self {
        Self {
            posture: RelationalComponentPlanPosture::RetainExact,
            expected,
            prepared_candidate: None,
        }
    }

    pub(crate) fn publish_prepared(
        expected: AdmittedRelationalBranchBasis,
        prepared_candidate: PreparedRelationalCommitCandidate,
    ) -> Self {
        Self {
            posture: RelationalComponentPlanPosture::PublishPrepared,
            expected,
            prepared_candidate: Some(prepared_candidate),
        }
    }

    pub(crate) fn take_prepared_candidate(&mut self) -> Option<PreparedRelationalCommitCandidate> {
        self.prepared_candidate.take()
    }
}
