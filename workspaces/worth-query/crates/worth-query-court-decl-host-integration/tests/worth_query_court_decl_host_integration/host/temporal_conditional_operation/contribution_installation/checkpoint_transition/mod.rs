pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::checkpoint_transition::*;
use worth_query_testkit_document_retention::facade::directory_home::{
    unique_directory, with_directory_bootstrap, DirectoryHomeGuard,
};

#[test]
fn checkpoint_transition_refuses_ordinary_reopen_and_guards_signal_programme_migration() {
    let dir = unique_directory("temporal-program-transition");
    let predecessor = DirectoryHomeGuard::wrap(open_predecessor_directory(dir.clone()), dir);
    let dir = predecessor.checkpoint_and_close_for_reopen();

    let ordinary_refused =
        with_directory_bootstrap(|phase| successor_request(dir.clone()).open_in_advancement(phase))
            .err()
            .expect("ordinary successor open must refuse the predecessor activation");
    assert!(
        ordinary_refused.transition_required().is_some(),
        "expected TransitionRequired, got {ordinary_refused:?}"
    );
    drop(ordinary_refused);

    // D16 Directory Home program transition currently supports Relational-only
    // programme transitions; TemporalHostSchema carries a Signal conditional
    // contribution and must refuse cleanly before invoking the author closure.
    let mut author_invoked = false;
    let refused = with_directory_bootstrap(|phase| {
        successor_request(dir.clone())
            .with_program_transition(
                predecessor_request(dir.clone()),
                transition_resources(),
                |_source, _target| {
                    author_invoked = true;
                    unreachable!("Signal programme transition refuses before authoring")
                },
            )
            .open_in_advancement(phase)
    })
    .err()
    .expect("D16 transition must refuse Signal programme migration before authoring");
    assert!(
        !author_invoked,
        "author closure must not execute when Signal programme migration is refused"
    );
    assert!(
        format!("{refused:?}")
            .contains("authored Signal programme migration requires its later migration phase"),
        "expected explicit Signal programme migration refusal, got {refused:?}"
    );
    drop(refused);

    // Predecessor Directory Home remains completely intact after both refusals.
    let reopened = DirectoryHomeGuard::wrap(open_predecessor_directory(dir.clone()), dir);
    assert!(reopened
        .with_host_advancement(|host_phase| reopened
            .on_branch(reopened.current_world())
            .select(&host_phase)
            .is_ok())
        .expect("installed host call opens"));
}
