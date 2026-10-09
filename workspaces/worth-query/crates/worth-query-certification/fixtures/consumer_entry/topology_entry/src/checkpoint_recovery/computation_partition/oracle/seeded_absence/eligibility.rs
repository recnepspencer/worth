//! Prior eligibility from the declared read alphabet and the model's outcome.
use super::*;
pub(super) enum Eligibility {
    Absent(Cause),
    Retained(Model),
}
impl Eligibility {
    pub(super) fn after<const WORK: usize, const RUNS: usize, const MODE: u8>(
        model: &Model,
    ) -> Self {
        if RUNS > 1 {
            return Self::Absent(Cause::SeveralComputations);
        }
        let completed = if MODE == 1 {
            model.observation_keys()
        } else {
            model.clone()
        }
        .completes_at(WORK);
        if completed {
            Self::Retained(model.clone())
        } else {
            Self::Absent(Cause::Stopped)
        }
    }
    // Canonical input framing is admitted before comparison; these literal
    // domain/identity bytes alone exhaust the margin of the two-wide-item prior.
    pub(super) fn expected<const MODE: u8, const WORK: usize>(
        &self,
        now: &Model,
        index: usize,
    ) -> Run {
        if index > 0 {
            return Run::Full(Cause::NoPriorHanded);
        }
        match self {
            Self::Absent(cause) => Run::Full(*cause),
            Self::Retained(old) if old.odd != now.odd => Run::Full(Cause::InputChanged),
            Self::Retained(old)
                if old.comparison_bound(MODE == 1)
                    + 16
                    + "worth-query.computation-input-value.v1".len()
                    + "checkpoint-region-entries".len()
                    > WORK =>
            {
                Run::Full(Cause::ObservationOverBudget)
            }
            Self::Retained(_) => Run::Incremental,
        }
    }
}
pub(super) fn assert_report(run: &OracleRun, expected: Run) {
    let full = match expected {
        Run::Full(cause) => vec![cause],
        Run::Incremental => vec![],
    };
    assert_eq!(
        run.full_preparations, full,
        "typed preparation cause, failed runs included"
    );
    if run.outcome.is_ok() {
        assert_eq!(run.runs, [expected]);
    } else {
        assert!(run.runs.is_empty(), "a failed run has no completion report");
    }
}

/// A fresh prime's typed cause and calls come from the independently seeded model.
pub(super) fn assert_prime<const WORK: usize, const MODE: u8>(runs: &[OracleRun], model: &Model) {
    for (index, run) in runs.iter().enumerate() {
        assert_report(
            run,
            Run::Full(if index == 0 {
                Cause::FirstRun
            } else {
                Cause::NoPriorHanded
            }),
        );
        assert_eq!(
            run.calls,
            if MODE == 1 {
                model.expected_observation_calls_at(None, WORK)
            } else {
                model.expected_calls_at(None, WORK)
            },
            "absence prime"
        );
    }
}
