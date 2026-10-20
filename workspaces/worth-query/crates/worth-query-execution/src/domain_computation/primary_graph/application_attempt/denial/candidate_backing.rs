use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};

impl Denial {
    pub(in crate::domain_computation::primary_graph) fn candidate_memory_denied(
        cause: worth_execution::MemoryLimitDenial,
    ) -> Self {
        Self {
            kind: Kind::AllocationDenied,
            subject: "candidate backing request memory".to_owned(),
            cause: AttemptDenialCause::CandidateMemory(cause),
        }
    }

    /// Original request capacity or accounting-overflow refusal for effect backing.
    pub fn candidate_memory_denial(&self) -> Option<worth_execution::MemoryLimitDenial> {
        match self.cause {
            AttemptDenialCause::CandidateMemory(cause) => Some(cause),
            _ => None,
        }
    }
}
