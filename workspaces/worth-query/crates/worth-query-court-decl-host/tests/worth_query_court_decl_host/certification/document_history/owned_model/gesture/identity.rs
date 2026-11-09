//! Dynamic membership keeps the original ordered-list meaning and installed bounds.
use super::*;

fn pair<'r>(
    document: world::Document<'r>,
    host: &world::Host,
    request: &Request<'_, '_, '_>,
    reverse: bool,
) -> Gesture<'r> {
    let mut group = open(document, host, request, 5410);
    if reverse {
        group
            .push::<_, MemberAction>(request, member(202), "child".into())
            .unwrap();
        group
            .push::<_, ModelAction>(request, model(101), "model".into())
            .unwrap();
    } else {
        group
            .push::<_, ModelAction>(request, model(101), "model".into())
            .unwrap();
        group
            .push::<_, MemberAction>(request, member(202), "child".into())
            .unwrap();
    }
    group
}
#[test]
fn dynamic_replay_preserves_the_selected_step_and_changed_member_order_refuses() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut group = pair(world::attach(&host), &host, &request, false);
    drive(&mut group, &request, |group| {
        group.posture() == WorthQueryHistoryGroupPosture::ReadyToComplete
    });
    let (document, execution) = group.parts();
    let step = execution.complete(document, &request).unwrap();
    let finished = group
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    drop(finished.completions);
    let before = finished.document.selection(&request).unwrap();
    let mut replay = pair(finished.document, &host, &request, false);
    drive(&mut replay, &request, |group| {
        group.posture() == WorthQueryHistoryGroupPosture::AlreadyRecorded
    });
    let finished = replay
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(finished.step, step);
    assert_eq!(
        finished.kind,
        WorthQueryHistoryGroupCompletionKind::AlreadyRecorded
    );
    assert!(finished.completions.is_empty());
    assert_eq!(finished.document.selection(&request).unwrap(), before);
    let mut changed = pair(finished.document, &host, &request, true);
    let (document, execution) = changed.parts();
    execution.advance(document, &request).unwrap();
    let cause = execution.advance(document, &request).unwrap_err();
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::IntentDrift
            ))
        ),
        "{cause:?}"
    );
    let document = changed
        .abort(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(document.selection(&request).unwrap(), before);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn dynamic_member_limit_precedes_capture_and_preserves_the_described_prefix() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut group = open(world::attach(&host), &host, &request, 5411);
    let before = group.document().selection(&request).unwrap();
    for index in 0..64 {
        if index % 2 == 0 {
            group
                .push::<_, ModelAction>(&request, model(index), format!("model:{index}"))
                .unwrap();
        } else {
            group
                .push::<_, MemberAction>(&request, member(index), format!("member:{index}"))
                .unwrap();
        }
    }
    let cause = group
        .push::<_, ModelAction>(&request, model(3), "over-limit".into())
        .unwrap_err();
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::GroupMemberLimitExceeded
            ))
        ),
        "{cause:?}"
    );
    assert_eq!(group.posture(), WorthQueryHistoryGroupPosture::Constructing);
    assert_eq!(group.completed_members(), 0);
    assert_eq!(group.document().selection(&request).unwrap(), before);
    let document = group
        .abort(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(document.selection(&request).unwrap(), before);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn dynamic_result_limit_refuses_a_second_type_before_any_effects() {
    let host = world::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let document = world::attach_as::<ResultLimitedHistory>(&host);
    let before = document.selection(&request).unwrap();
    let mut group = document
        .gesture(&host, &request, 5412)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    group
        .push::<_, ModelAction>(&request, model(3), "model".into())
        .unwrap();
    let cause = group
        .push::<_, MemberAction>(&request, member(5), "member".into())
        .unwrap_err();
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::GroupResultLimitExceeded
            ))
        ),
        "{cause:?}"
    );
    assert_eq!(group.document().selection(&request).unwrap(), before);
    let document = group
        .abort(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert_eq!(document.selection(&request).unwrap(), before);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
