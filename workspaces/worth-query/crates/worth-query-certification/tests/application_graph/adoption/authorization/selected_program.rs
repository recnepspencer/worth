//! The currently activated source chooses permission, never the target or initial host program.
use super::*;
use crate::document_retention_model::{
    host::publish,
    programs::{validated_first_program, validated_second_program},
    schema::{DocumentRetentionContribution, DocumentRetentionSchema},
};
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
use worth_query_host::facade::declaration::application_program::*;
struct NoAdoptionProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for NoAdoptionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.no-program-adoption.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        RetentionProgramP0::feature_specs()
    }
}
fn no_adoption_program() -> ValidatedApplicationProgram<DocumentRetentionSchema, NoAdoptionProgram>
{
    ApplicationProgramAuthoring::begin()
        .validated_program()
        .unwrap()
}
#[test]
fn activated_program_without_declared_capability_refuses_even_with_real_grants() {
    let host = publish(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(no_adoption_program()),
    )
    .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let target = *host
        .supported_program::<NoAdoptionProgram>()
        .unwrap()
        .owned_revision();
    let programs = host.runtime().request(&principal, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    assert!(matches!(
        programs
            .adopt(&requirements)
            .unwrap()
            .prepare(64)
            .unwrap()
            .publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    match host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .compare(&target)
        .err()
        .unwrap()
    {
        WorthQueryApplicationProgramAdoptionPreparationDenial::Authorization(
            WorthQueryProgramAdoptionAuthorizationDenial::CapabilityNotDeclared,
        ) => {}
        other => panic!("activated source has no declared permission: {other:?}"),
    }
}
#[test]
fn target_without_permission_cannot_authorize_its_own_adoption() {
    let host = publish(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(no_adoption_program()),
    )
    .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let target = *host
        .supported_program::<NoAdoptionProgram>()
        .unwrap()
        .owned_revision();
    let requirements = host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .compare(&target)
        .unwrap();
    revoke(&host);
    match host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .adopt(&requirements)
        .err()
        .unwrap()
    {
        WorthQueryApplicationProgramAdoptionPreparationDenial::Authorization(cause) => {
            permission_denied(cause)
        }
        other => panic!("source still governs: {other:?}"),
    }
    assert_eq!(
        host.runtime()
            .request(&principal, &scope)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        host.owned_revision()
    );
}
