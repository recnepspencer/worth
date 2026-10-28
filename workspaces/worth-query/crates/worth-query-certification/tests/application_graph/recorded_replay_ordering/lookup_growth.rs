//! Recorded replay lookup remains independent of retained population.
use super::*;

fn populated(population: u64) -> DocumentRetentionRuntime<RetentionProgramP0> {
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    for offset in 0..population {
        let key = 0x5e92_0000 + offset;
        let (committed, _) = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(host.current_world())
            .advanced()
            .mutate(SetRetentionIntent {
                input: SetRetentionInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                    // Alternate the value so every key records a real change.
                    retention_days: RETENTION + offset % 2,
                },
            })
            .without_source()
            .idempotency(&key)
            .execute_in_program_report(&host, Policy::SystemAllocation)
            .into_parts();
        assert!(matches!(
            committed.expect("each population key commits"),
            Outcome::Committed { .. }
        ));
    }
    host
}

/// One replay of the oldest recorded key under a stopped lease, timed.
fn timed_replay(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    stopped: &ExecutionResourceLease<'_>,
) -> Duration {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .advanced()
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: RETENTION,
            },
        })
        .without_source();
    let key = 0x5e92_0000;
    let started = Instant::now();
    let (replay, work) = request
        .idempotency(&key)
        .execute_in_program_report(host, Policy::Execution(stopped))
        .into_parts();
    let elapsed = started.elapsed();
    assert!(matches!(
        replay.expect("the oldest recorded key replays"),
        Outcome::AlreadyCommitted(_) | Outcome::PreviouslyCommitted(_)
    ));
    assert_eq!(work, WorthQueryMutationHandlerWork::NotStarted);
    elapsed
}

/// A growth class, not a count: thirty-two times the recorded outcomes must
/// not make the World-side pre-admission step grow with them. Samples of the
/// two hosts alternate, so machine load lands on both, and each side keeps its
/// fastest sample, since noise only adds. A step that walked the records would
/// scale toward 32x; the bound leaves room for noise without admitting that.
#[test]
fn the_pre_admission_step_does_not_grow_with_recorded_outcomes() {
    let authority =
        worth_query_host::facade::primary_graph::query_test_execution_authority_for_test();
    let stopping = CancellationSource::new();
    let stopped = authority
        .request_lease(lease_request(0, stopping.token()))
        .unwrap();
    stopping.cancel();
    let small = populated(4);
    let large = populated(128);
    let (mut small_floor, mut large_floor) = (Duration::MAX, Duration::MAX);
    for _ in 0..REPLAY_SAMPLES {
        small_floor = small_floor.min(timed_replay(&small, &stopped));
        large_floor = large_floor.min(timed_replay(&large, &stopped));
    }
    assert!(
        large_floor <= small_floor.saturating_mul(4),
        "replay floor grew from {small_floor:?} at 4 recorded outcomes to {large_floor:?} at 128"
    );
}
