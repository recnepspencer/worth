//! Recorded replay lookup remains independent of retained population.
pub use super::*;
pub use std::time::Instant;

pub fn populated(population: u64) -> DocumentRetentionRuntime<RetentionProgramP0> {
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
pub fn timed_replay(
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
