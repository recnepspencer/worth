//! Public inspection and fail-closed admission for semantic program impact.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

use crate::document_retention_model::host::{publish_on_first_program, SEED_RETENTION};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::programs::ChangedFeatureRetentionProgram;
use crate::document_retention_model::readback::read_retention;

#[test]
fn migration_assessment_impact_is_inspectable_and_stops_before_selection() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<ChangedFeatureRetentionProgram>()
        .expect("the changed-feature target is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs
        .compare(&target)
        .expect("the public comparison exposes semantic impact");
    assert!(requirements.requires_migration_assessment());
    assert!(requirements.semantic_diff().changes().iter().any(|change| {
        change.family()
            == worth_query_host::facade::declaration::application_program::ApplicationSemanticFamily::Features
    }));

    let denial = match programs.adopt(&requirements).prepare(0) {
        Ok(_) => panic!("migration assessment must fail before zero-budget selection"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial::MigrationAssessmentRequired(_)
        )
    ));
    assert_eq!(read_retention(host.runtime(), branch), SEED_RETENTION);
}
