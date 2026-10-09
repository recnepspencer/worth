//! Unretained decisions have no published state and preserve exact absence.
use super::*;
pub(super) fn unretained<
    const REUSE: bool,
    const WORK: usize,
    const RUNS: usize,
    const MODE: u8,
>(
    application: &Application<REUSE, WORK, RUNS, MODE>,
    fresh: &Application<REUSE, WORK, RUNS, MODE>,
    model: &Model,
    command: &mut u64,
) {
    let mut outcomes = Vec::new();
    for application in [application, fresh] {
        let (scope, principal) = authenticate(application);
        let request = application.request(&principal, &scope);
        owner::take_outcomes();
        owner::calls::take();
        published_states();
        discarded_retention();
        let observed = request
            .query(PlanarRead {
                body_key: SCOPE.to_owned(),
            })
            .execute()
            .unwrap();
        *command += 1;
        let committed = request
            .mutate(super::super::super::demand::RegionTotalsDemand {
                scope_key: SCOPE.to_owned(),
                entries: if model.odd { "odd" } else { "even" }.to_owned(),
                replacement_y: length(if model.odd { ODD_Y } else { EVEN_Y }),
            })
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(command)
            .execute_in_program::<OracleProgram<REUSE, WORK, RUNS, MODE>>(
                application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(
                committed,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "unretained mutation must commit: {committed:?}"
        );
        let calls = owner::calls::take();
        let expected = model.expected_calls(None);
        assert_eq!(
            calls,
            [
                expected.plans,
                expected.keys,
                expected.gathers,
                expected.kernels
            ],
            "every unretained decision enters the full ordinary owner exactly once"
        );
        let outcome = owner::take_outcomes();
        assert_eq!(outcome.len(), 1);
        outcomes.push(outcome);
        // An ordinary mutation has no producer row. Its computed result is
        // configured Unretained at installation: policy suppresses retention
        // before prior delivery, and it publishes no state.
        assert!(
            published_states().is_empty(),
            "no producer row is published"
        );
        assert_eq!(discarded_retention(), [Some(Cause::RetentionPolicy)]);
    }
    // Both runtimes publish no state and discard the exact typed result,
    // while reporting the same work and named partition stop.
    assert_eq!(
        outcomes[0], outcomes[1],
        "unretained outcome, work and named stop"
    );
}
