//! Original retries and installed count limits cannot append partial groups.
use super::*;

#[test]
fn original_group_retry_returns_its_one_step_and_changed_membership_refuses() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut group = begin(&host, document, 5, 0x9182_5400);
    until(&host, &mut group, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    let step = {
        let (document, execution) = group.parts();
        execution.complete(document, &request).unwrap()
    };
    let finished = group.finish(&request).unwrap();
    drop(finished.completions);
    let undo = super::super::navigation::complete(
        &host,
        finished
            .document
            .navigate::<Navigation>()
            .undo_step()
            .expected_selection(finished.document.selection(&request).unwrap())
            .idempotency(0x9182_5401)
            .prepare(&request)
            .unwrap(),
    );
    assert_eq!(undo.selection().step().id(), step.parent().unwrap());
    assert_eq!(value(&host, &finished.document), 7);
    let before = finished.document.selection(&request).unwrap();
    let mut replay = begin(&host, finished.document, 5, 0x9182_5400);
    until(&host, &mut replay, |group| {
        group.posture() == Posture::AlreadyRecorded
    });
    let replay = replay.finish(&request).unwrap();
    assert_eq!(replay.step, step);
    assert_eq!(replay.kind,worth_query_host::facade::application_entry::WorthQueryHistoryGroupCompletionKind::AlreadyRecorded);
    assert!(
        replay.completions.is_empty(),
        "no repeated source operation is invented"
    );
    assert_eq!(replay.document.selection(&request).unwrap(), before);
    let mut drift = begin(&host, replay.document, 6, 0x9182_5400);
    let (document, execution) = drift.parts();
    assert_eq!(
        execution.advance(document, &request).unwrap(),
        Posture::Admitting
    );
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
    let document = drift.abort(&request).unwrap();
    assert_eq!(document.selection(&request).unwrap(), before);
    close(&host, document, 3);
}

#[test]
fn installed_member_capacity_refuses_before_capturing_or_executing_a_partial_group() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let members = std::array::from_fn::<_, 65, _>(|index| (input(3), 0x9182_5500 + index as u64));
    let (document, cause) = match document.group_edits::<_, RetentionProgramP0, Action, 65>(
        &host,
        &request,
        0x9182_5500,
        members,
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("installed finite member count must refuse"),
    };
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::GroupMemberLimitExceeded
            ))
        ),
        "{cause:?}"
    );
    assert_eq!(document.selection(&request).unwrap(), before);
    assert_eq!(value(&host, &document), 7);
    let mut lawful = begin(&host, document, 5, 0x9182_5500);
    until(&host, &mut lawful, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    {
        let (document, execution) = lawful.parts();
        execution.complete(document, &request).unwrap();
    }
    let finished = lawful.finish(&request).unwrap();
    drop(finished.completions);
    close(&host, finished.document, 2);
}
