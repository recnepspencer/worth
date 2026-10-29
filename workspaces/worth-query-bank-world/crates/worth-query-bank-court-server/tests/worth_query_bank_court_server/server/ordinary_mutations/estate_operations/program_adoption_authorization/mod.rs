pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::program_adoption_authorization::*;

#[test]
fn authenticated_actor_without_explicit_deployment_grant_cannot_compare_or_adopt() {
    let fixture = notification_world("deployment-grant-missing", DeathNoticeStatus::Verified);
    let actor = fixture.authenticate_deceased();
    let authorized = fixture.authenticate_specialist();
    let scope = request_scope();
    let runtime = &fixture.world.runtime;
    let source = *runtime
        .inspect_branch_program(&authorized, &scope, runtime.current_branch())
        .unwrap()
        .revision();
    let target = runtime
        .supported_program_revision::<BankApplicationP1>()
        .unwrap();
    let requirements = runtime
        .request(&authorized, &scope)
        .programs()
        .compare(&target)
        .unwrap();
    let programs = runtime.request(&actor, &scope).programs();
    permission_denied(
        programs.compare(&target).err().unwrap(),
        WorthQueryOperationAuthorizationRefusalKind::CapabilityGrantMissing,
    );
    permission_denied(
        programs.adopt(&requirements).err().unwrap(),
        WorthQueryOperationAuthorizationRefusalKind::CapabilityGrantMissing,
    );
    assert_eq!(
        runtime
            .inspect_branch_program(&authorized, &scope, runtime.current_branch())
            .unwrap()
            .revision(),
        &source
    );
}

#[test]
fn explicit_source_grant_authorizes_adoption_but_not_the_next_source_revision() {
    let fixture = notification_world(
        "deployment-grant-source-affinity",
        DeathNoticeStatus::Verified,
    );
    let actor = fixture.authenticate_specialist();
    let scope = request_scope();
    let runtime = &fixture.world.runtime;
    let target = runtime
        .supported_program_revision::<BankApplicationP1>()
        .unwrap();
    let programs = runtime.request(&actor, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(128).unwrap();
    let performed = runtime.request(&actor, &scope).programs().publish(prepared);
    assert!(matches!(
        performed,
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    assert_eq!(
        runtime
            .inspect_branch_program(&actor, &scope, runtime.current_branch())
            .unwrap()
            .revision(),
        &target
    );
    let next = runtime
        .supported_program_revision::<BankApplicationP2>()
        .unwrap();
    permission_denied(
        runtime
            .request(&actor, &scope)
            .programs()
            .compare(&next)
            .err()
            .unwrap(),
        WorthQueryOperationAuthorizationRefusalKind::CapabilityGrantMissing,
    );
    assert_eq!(
        runtime
            .inspect_branch_program(&actor, &scope, runtime.current_branch())
            .unwrap()
            .revision(),
        &target
    );
}

#[test]
fn issuer_revocation_between_prepare_and_publication_preserves_native_reservation() {
    let fixture = notification_world("deployment-grant-revocation", DeathNoticeStatus::Verified);
    let actor = fixture.authenticate_specialist();
    let scope = request_scope();
    let runtime = &fixture.world.runtime;
    let source = *runtime.installed_program_revision();
    let target = runtime
        .supported_program_revision::<BankApplicationP1>()
        .unwrap();
    let programs = runtime.request(&actor, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(128).unwrap();
    revoke(&fixture);
    permission_denied(
        runtime
            .request(&actor, &scope)
            .programs()
            .compare(&target)
            .err()
            .unwrap(),
        WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing,
    );
    let prepared = match runtime.request(&actor, &scope).programs().publish(prepared) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
            permission_denied(
                WorthQueryApplicationProgramAdoptionPreparationDenial::Authorization(cause),
                WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing,
            );
            prepared
        }
        _ => panic!("the revoked actual grant must preserve the native prepared owner"),
    };
    assert_eq!(prepared.source(), &source);
    assert_eq!(prepared.target(), &target);
    assert_eq!(
        runtime
            .inspect_branch_program(&actor, &scope, runtime.current_branch())
            .unwrap()
            .revision(),
        &source
    );
}

#[test]
fn a_non_issuer_cannot_revoke_the_deployment_grant() {
    let fixture = notification_world(
        "deployment-grant-revocation-issuer",
        DeathNoticeStatus::Verified,
    );
    let actor = fixture.authenticate_deceased();
    let scope = request_scope();
    let runtime = &fixture.world.runtime;
    let result = {
        let actor = &actor;
        let scope = &scope;
        let operation=runtime.retain_program_deployment_revocation(actor,bank_domain::schema::ProgramDeploymentRevocationInput { grant_identity: ("specialist-p0-deployment").into() },91764012_u64,worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions::new(),scope).unwrap_or_else(|cause| panic!("declared issuer revocation: {cause:?}"));
        operation.advance(&runtime.request(actor, scope))
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: _original_revocation, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Authorization(denial)) } = result else { panic!("recorded issuer permission must refuse before effect") };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
        )
    );
    let issuer = fixture.authenticate_specialist();
    let target = runtime
        .supported_program_revision::<BankApplicationP1>()
        .unwrap();
    assert!(runtime
        .request(&issuer, &scope)
        .programs()
        .compare(&target)
        .is_ok());
}
