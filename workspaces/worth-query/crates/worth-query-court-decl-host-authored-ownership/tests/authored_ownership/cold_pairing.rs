//! A cold region authenticates presence even for an unrelated access within it.
use super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
    *,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    publication::AuthoredRetirementRefusal,
    transactions::{RecordRef, TransactionCommitError},
};
#[test]
fn external_cold_pairing_rejects_whole_fact_family_loss_before_an_unrelated_access() {
    type S = Model<6>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let created = create(&host, &request, 40, vec![]);
    let sentinel = created
        .outputs_of::<super::lifecycle::CreatedOutputs>()
        .unwrap()
        .entity::<super::lifecycle::Created<S>>()
        .unwrap()
        .entity_id();
    let original = super::current_fence::denied(&host);
    let Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { publication, owner }) =
        original.authored_retirement_refusal()
    else {
        panic!("{original:?}");
    };
    let results = host
        .runtime()
        .probe_cold_current_ownership_pairing_for_test(
            host.runtime().current_world(),
            owner,
            sentinel,
            &scope,
        )
        .unwrap()
        .expect("actual Query publication recaptures and recovers cold");
    assert_eq!(results.len(), 2);
    for result in results {
        let error = result.expect_err(
            "a cold current member cannot silently lose either independent fact family",
        );
        let TransactionCommitError::Publication { error, .. } = error else {
            panic!("{error:?}");
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::CorruptCurrentOwnership { publication })
        );
        assert_eq!(
            error.context.affected_records.first(),
            Some(&RecordRef::Entity(owner))
        );
    }
}
