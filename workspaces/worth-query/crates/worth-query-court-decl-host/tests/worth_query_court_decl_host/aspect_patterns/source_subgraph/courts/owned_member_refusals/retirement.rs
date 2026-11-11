//! Ordinary roster refusal setup and typed denial projection.
use super::*;
pub(super) fn retire(
    host: &world::Host,
    request: &world::Request<'_, '_, '_>,
    input: Retire,
) -> WorthQueryApplicationMutationOutcome<edit::Denial, edit::Edited> {
    request
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&"refuse-roster".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}
pub(super) fn denied(
    outcome: WorthQueryApplicationMutationOutcome<edit::Denial, edit::Edited>,
) -> WorthQueryApplicationCommitDenial {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("{outcome:?}");
    };
    denial
}
pub(super) fn command(count: usize) -> Retire {
    Retire {
        key: "roster".into(),
        members: image(count)
            .members
            .into_iter()
            .map(|row| row.key)
            .collect(),
        replacement: None,
    }
}
