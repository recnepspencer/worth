//! A handler's read of the trusted admission instant is request context: the
//! decision depends on when it ran, so it is never reused for another request.
use std::cell::Cell;

use super::DecisionContextUse;

pub(super) type AdmittedAt = Result<
    worth_foundational::facade::CanonicalTimestamp,
    worth_query_execution_contracts::facade::runtime_time::WorthQueryRuntimeTimeDenial,
>;

/// Records the read before lending the operation's single instant.
pub(super) fn observe(
    context_use: &Cell<DecisionContextUse>,
    admitted_at: AdmittedAt,
) -> AdmittedAt {
    context_use.set(context_use.get().request_context());
    admitted_at
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::CanonicalTimestamp;

    #[test]
    fn reading_the_admission_instant_marks_untracked_request_context() {
        let context_use = Cell::new(DecisionContextUse::default());
        assert!(context_use.get().supports_candidate_planning());
        let instant = CanonicalTimestamp {
            micros_since_unix_epoch: 1_795_829_400_000_000,
        };
        assert_eq!(observe(&context_use, Ok(instant)), Ok(instant));
        assert!(context_use.get().has_untracked_context());
        assert!(!context_use.get().supports_candidate_planning());
        assert_eq!(
            context_use.get(),
            DecisionContextUse::default().request_context()
        );
    }
}
