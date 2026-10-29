pub(crate) use {std::sync::atomic::Ordering};
pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_query::facade::foundation::QueryExternalIdentityToken, worth_query::facade::foundation::WorthQueryEntityIdentity};
pub(crate) use {worth_query::facade::runtime::WorthQueryAspectMutationBuilder, worth_query::facade::runtime::WorthQueryExistingEntityTarget, worth_query::facade::runtime::WorthQueryExistingTruthBindingAuthorityLabel, worth_query::facade::runtime::WorthQueryExistingTruthTargetBinding, worth_query::facade::runtime::WorthQueryMutationAuthorityIdentity, worth_query::facade::runtime::WorthQueryRuntimeBackendPosture, worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily, worth_query::facade::runtime::WorthQueryRuntimeFamilySupport, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile, worth_query::facade::runtime::WorthQueryWriteCommand};
pub(crate) use {worth_server::WorthServerDirectMutationOutcome, worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerQueryOperation, worth_server::WorthServerResponseInput, worth_server::WorthServerSuccessKind};

pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_real_mutation::RealMutationWorkspaceProvider};
pub(crate) use {worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest, worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record, worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest, worth_testkit_server::shared_support_worth_native_assertions::worth_native_session};
pub(crate) use {worth_testkit_server::shared_support_worth_native_runtime::build_server, worth_testkit_server::shared_support_worth_native_runtime::build_server_with_profiled_counting_workspace};

pub(crate) fn prepared_compat_mutation_request(
    server: &worth_server::WorthServer,
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Mutation)
            .with_method("POST")
            .with_path(format!("/compat/mutations/{operation_name}"))
            .with_header("accept", "application/json")
            .with_body_content_type("application/json")
            .with_body_present(true)
            .build()
            .expect("compat mutation request should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(prepared) => prepared,
        other => panic!("expected prepared compat mutation request, got {other:?}"),
    }
}

pub(crate) fn build_dual_surface_mutation_server() -> worth_server::WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(RealMutationWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("dual-surface mutation server should build")
}

pub(crate) fn compat_mutation_success(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> worth_server::WorthServerCompatibilityMutation {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility mutation success, got {other:?}"),
    }
}

pub(crate) fn insert_task(identity: &str) -> WorthQueryWriteCommand {
    worth_query::facade::runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", identity)
        .aspect("title.value", format!("Title for {identity}"))
        .build_insert("Task")
        .expect("insert command should build")
}

pub(crate) fn verify_existing_task(
    authoritative_identity: &str,
    resolved_entity_identity: &str,
) -> WorthQueryWriteCommand {
    worth_query::facade::runtime::WorthQueryAspectMutationBuilder::new()
        .aspect("title.value", "Expected title")
        .build_verify_existing(existing_binding(
            authoritative_identity,
            resolved_entity_identity,
        ))
        .expect("verify existing command should build")
}

pub(crate) fn existing_binding(
    authoritative_identity: &str,
    resolved_entity_identity: &str,
) -> WorthQueryExistingTruthTargetBinding {
    worth_query::facade::runtime::WorthQueryExistingTruthTargetBinding::from_entity_target(
        worth_query::facade::runtime::WorthQueryExistingEntityTarget::new(
            worth_query::facade::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
                worth_query::facade::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new(authoritative_identity)
                    .expect("existing-truth authority label"),
            )
            .expect("existing-truth authority identity"),
            worth_query::facade::foundation::WorthQueryEntityIdentity::admit_authored_entity_token(worth_query::facade::foundation::QueryExternalIdentityToken::new(
                std::sync::Arc::from(resolved_entity_identity),
            )),
        )
        .expect("existing entity target should build")
        .in_target_collection("Task")
        .expect("existing entity target collection should build"),
    )
    .expect("existing entity binding should build")
}

pub(crate) fn direct_mutation_success(
    outcome: WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerDirectMutation {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct mutation, got {other:?}"),
    }
}

pub(crate) fn direct_mutation_denied(
    outcome: WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct mutation, got {other:?}"),
    }
}