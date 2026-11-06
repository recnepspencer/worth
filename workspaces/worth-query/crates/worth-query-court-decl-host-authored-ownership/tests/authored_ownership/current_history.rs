//! Existing history readers refuse an opted-in route whose current facts vanished.
use super::{
    correction_courts::install_definition,
    coupled::{
        history::reopen,
        multi::{create, settle, TwoProgram},
    },
    *,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    publication::AuthoredRetirementRefusal as Cause,
    transactions::{RecordRef, TransactionCommitError},
};
#[test]
fn published_owned_history_cannot_replace_missing_current_presence_on_either_route_door() {
    type S = Model<6>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    create(&host, &request, 40, vec![]);
    let expected = super::current_fence::denied(&host);
    let Some(Cause::OwnedPublicationRequiresCascade { publication, owner }) =
        expected.authored_retirement_refusal()
    else {
        panic!("{expected:?}")
    };
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    let damaged = super::coherent_roster::corrupt_current(&checkpoint, publication, 5);
    let recovered = reopen::<S>(damaged);
    let failures = recovered
        .runtime()
        .probe_current_history_contradiction_for_test(publication, owner, &scope)
        .unwrap()
        .unwrap();
    assert_eq!(failures.len(), 2);
    for failure in failures {
        let Err(TransactionCommitError::Publication { error, .. }) = failure else {
            panic!("missing current facts must refuse before history can stand in for them")
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(Cause::CorruptCurrentOwnership { publication })
        );
        assert_eq!(error.context.affected_records, [RecordRef::Entity(owner)]);
    }
    let principal = authenticate_operator(recovered.installed_schema(), &scope);
    let request = recovered.runtime().request(&principal, &scope);
    assert_eq!(
        request
            .query(query::Input { id: 40 })
            .execute()
            .unwrap()
            .rows()
            .len(),
        1
    );
}
