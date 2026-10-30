pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::authorization::selected_program::*;

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
