//! Count current ownership reach beside cold, unrelated physical partitions.
use super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
    lifecycle::*,
    *,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::publication::AuthoredRetirementRefusal;
#[test]
fn current_custody_contacts_only_the_touched_publication_and_leaves_other_regions_cold() {
    type S = Model<6>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let owner = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    let count = host
        .runtime()
        .probe_current_ownership_locality_for_test(created.product_branch(), owner, &scope)
        .unwrap();
    assert_eq!(
        count.cold_before, 63,
        "the ordinary source-edit head reopens without unrelated members"
    );
    assert_eq!(
        count.unrelated_roster_records, 0,
        "unrelated edits contact no custody record"
    );
    assert_eq!(count.cold_after_unrelated, count.cold_before);
    assert_eq!(
        count.owned_publications, 1,
        "an owned edit validates only its own publication"
    );
    assert_eq!(count.cold_after_owned, count.cold_before);
    assert!(
        matches!(count.owned_refusal.authored_retirement_refusal(),Some(AuthoredRetirementRefusal::OwnedPublicationRequiresCascade{owner:actual,..}) if actual==owner)
    );
}
