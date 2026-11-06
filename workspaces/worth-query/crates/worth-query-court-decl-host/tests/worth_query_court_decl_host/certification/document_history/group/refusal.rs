//! A real second invariant refusal preserves active state and the first staged commit.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome,
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted},
};

#[test]
fn second_member_invariant_refusal_keeps_the_actual_first_publication_private_until_abort() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut group = begin(&host, document, 11, 0x9182_5100);
    until(&host, &mut group, |group| group.completed_members() == 1);
    {
        let (document, execution) = group.parts();
        assert_eq!(value(&host, document), 7);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            3
        );
        let WorthQuerySourceCompletion::Fresh { result, .. } =
            execution.completed_source(0).unwrap().evidence()
        else {
            panic!("first actual completion")
        };
        assert_eq!(result.retention_days, 3);
    }
    until(&host, &mut group, |group| {
        group.posture() == Posture::Terminal
    });
    {
        let (document, execution) = group.parts();
        let Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::Denied(cause),
        )) = execution.terminal_source().unwrap().native_outcome()
        else {
            panic!("real invariant terminal")
        };
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
        );
        assert!(cause.custom_invariant_violation_identity().is_some());
        assert_eq!(value(&host, document), 7);
        assert_eq!(document.current_step(&request).unwrap(), root);
        assert_eq!(
            execution.read_staged(document, &request).unwrap().rows()[0].retention_days,
            3
        );
        assert!(matches!(
            execution.complete(document, &request),
            Err(WorthQueryHistoryGroupDenial::NotReady)
        ));
    }
    let document = group.abort(&request).unwrap();
    assert_eq!(value(&host, &document), 7);
    assert_eq!(document.current_step(&request).unwrap(), root);
    assert!(host.branches().pending_cleanup().is_empty());
    // Abort releases the real pending occurrence, not merely a UI flag.
    let mut lawful = begin(&host, document, 5, 0x9182_5100);
    until(&host, &mut lawful, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    {
        let (document, execution) = lawful.parts();
        execution.complete(document, &request).unwrap();
    }
    let finished = lawful.finish(&request).unwrap();
    assert_eq!(value(&host, &finished.document), 5);
    drop(finished.completions);
    close(&host, finished.document, 2);
}
