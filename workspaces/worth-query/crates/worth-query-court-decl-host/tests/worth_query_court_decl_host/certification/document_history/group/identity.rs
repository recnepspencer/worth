//! Same-model documents cannot substitute their selection cells at split receivers.
use super::*;

fn correspondence_refusal(cause: WorthQueryHistoryGroupDenial) {
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::ForeignRecording
            ))
        ),
        "{cause:?}"
    );
}

#[test]
fn group_execution_rejects_a_same_model_document_and_swapped_owning_roots() {
    let host = publish_on_first_program();
    let first = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let second = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut group = begin(&host, first, 5, 0x9182_5200);
    {
        let (_, execution) = group.parts();
        correspondence_refusal(execution.advance(&second, &request).unwrap_err());
        match execution.read_staged(&second, &request) {
            Err(cause) => correspondence_refusal(cause),
            Ok(_) => panic!("foreign document cannot read group candidate"),
        }
        correspondence_refusal(execution.complete(&second, &request).unwrap_err());
    }
    assert_eq!(group.posture(), Posture::Constructing);
    assert_eq!(value(&host, &second), 7);
    until(&host, &mut group, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    {
        let (document, execution) = group.parts();
        execution.complete(document, &request).unwrap();
    }
    let mut other = begin(&host, second, 5, 0x9182_5201);
    until(&host, &mut other, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    {
        let (document, execution) = other.parts();
        execution.complete(document, &request).unwrap();
    }
    {
        let (_, first) = group.parts();
        let (_, second) = other.parts();
        std::mem::swap(first, second);
    }
    let (returned, cause) = match group.finish(&request) {
        Err(failure) => failure,
        Ok(_) => panic!("swapped group cannot return another document's result"),
    };
    group = returned;
    correspondence_refusal(cause);
    let (returned, cause) = other.abort(&request).unwrap_err();
    other = returned;
    correspondence_refusal(cause);
    {
        let (_, first) = group.parts();
        let (_, second) = other.parts();
        std::mem::swap(first, second);
    }
    let first = group.finish(&request).unwrap();
    let second = other.finish(&request).unwrap();
    assert_eq!(value(&host, &first.document), 5);
    assert_eq!(value(&host, &second.document), 5);
    drop(first.completions);
    drop(second.completions);
    close(&host, first.document, 2);
    close(&host, second.document, 2);
}
