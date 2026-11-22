//! The opaque Ready payload cannot erase attribution while its prerequisite runs.
use super::*;
fn member() -> RequiredMember<()> {
    RequiredMember {
        ready: MemberInput::native(()),
        caller: false,
        caller_successor: false,
        resolved_successor: false,
        queued: None,
        contacts: 0,
        state: MemberState::Inspect,
        dependency_schedule: DependencySchedule::PendingConsumed,
    }
}
#[test]
fn a_suspended_caller_successor_retains_its_execution_attribution() {
    let mut caller = member();
    caller.caller = true;
    caller.caller_successor = true;
    caller.contacts = 2;
    let mut frontier = RequiredFrontier {
        members: vec![caller, member()],
    };
    assert!(frontier.wait_for(0, 1));
    assert!(matches!(frontier.members[0].state, MemberState::Waiting(1)));
    assert!(
        !frontier.awaken(),
        "an unfinished prerequisite cannot resume the caller"
    );
    frontier.members[1].state = MemberState::Current(None);
    assert!(frontier.awaken());
    let resumed = &frontier.members[0];
    assert!(matches!(resumed.state, MemberState::Inspect));
    assert!(resumed.caller && resumed.caller_successor);
    assert_eq!(resumed.contacts, 2);
}
