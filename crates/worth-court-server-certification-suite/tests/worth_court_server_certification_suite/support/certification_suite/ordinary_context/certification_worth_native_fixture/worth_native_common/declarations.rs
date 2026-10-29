pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily, worth_query::facade::runtime::WorthQueryRuntimeFamilySupport, worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection, worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
pub(crate) use {worth_server::request_context::DiagnosticRichnessProfile, worth_server::WorthServer, worth_server::WorthServerDirectProjectionRequest, worth_server::WorthServerQuerySupportPosture, worth_server::WorthServerRequestContext, worth_server::WorthServerResponseEnvelope, worth_server::WorthServerWorthNativeSession};

pub(crate) use {worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider, worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record, worth_testkit_server::shared_support_worth_native_runtime::build_server, worth_testkit_server::shared_support_worth_native_runtime::build_server_with_profiled_workspace, worth_testkit_server::shared_support_worth_native_runtime::build_server_with_workspace_provider, worth_testkit_server::shared_support_worth_native_runtime::server_with_request_context_default, worth_testkit_server::shared_support_worth_native_runtime::worth_native_session_input_builder};

pub(crate) use {crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle};

pub(crate) fn standard_server() -> WorthServer {
    build_server(true)
}

pub(crate) fn forensic_server() -> WorthServer {
    server_with_request_context_default(worth_server::request_context::DiagnosticRichnessProfile::Forensic)
}

pub(crate) fn remask_server() -> WorthServer {
    build_server_with_workspace_provider(worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider::new(projection_remask()), true)
}

pub(crate) fn runtime_backed_server() -> WorthServer {
    build_server_with_profiled_workspace(worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile())
}

pub(crate) fn durable_later_server() -> WorthServer {
    build_server_with_profiled_workspace(
        worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::DurableArtifacts,
                "durable restart and artifact reload stay deferred for direct certification lanes",
            ),
        ),
    )
}

pub(crate) fn worth_native_session_for_branch(
    server: &WorthServer,
    branch_id: Option<&str>,
) -> WorthServerWorthNativeSession {
    worth_native_session_for_target(server, None, branch_id)
}

pub(crate) fn worth_native_session_for_target(
    server: &WorthServer,
    workspace_id: Option<&str>,
    branch_id: Option<&str>,
) -> WorthServerWorthNativeSession {
    let mut builder = worth_native_session_input_builder();
    if let Some(workspace_id) = workspace_id {
        builder = builder.with_workspace_id(workspace_id);
    }
    if let Some(branch_id) = branch_id {
        builder = builder.with_branch_id(branch_id);
    }

    match server.worth_native().session(
        builder
            .build()
            .expect("Worth-native certification session input should validate"),
    ) {
        worth_proof::TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native certification session, got {other:?}"),
    }
}

pub(crate) fn direct_bundle(
    server: &WorthServer,
    request_context: &WorthServerRequestContext,
    response: WorthServerResponseEnvelope,
) -> WorthServerCertificationBundle {
    crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(request_context),
        response.clone(),
        operator_evidence_record(server, response),
    )
}

pub(crate) fn request_context_digest(request_context: &WorthServerRequestContext) -> String {
    format!(
        "principal={};tenant={};workspace={};branch={};diagnostics={:?}",
        request_context.authenticated_principal().principal_id(),
        request_context.workspace_target().tenant_id(),
        request_context.workspace_target().workspace_id(),
        request_context.branch_target().canonical_label(),
        request_context.diagnostics_profile(),
    )
}

pub(crate) fn support_posture_digest(posture: &WorthServerQuerySupportPosture) -> String {
    match posture {
        worth_server::WorthServerQuerySupportPosture::ProductIndependent { label } => {
            format!("product-independent:{label}")
        }
        worth_server::WorthServerQuerySupportPosture::PrimaryGraphApplicationSupported { basis_token } => {
            format!("primary-graph-application-supported:{basis_token}")
        }
        worth_server::WorthServerQuerySupportPosture::QueryReadSupported { family_contract } => {
            format!("query-read-supported:{}", family_contract.contract_digest())
        }
        worth_server::WorthServerQuerySupportPosture::DirectReadSupported { family_contract } => {
            format!(
                "direct-read-supported:{}",
                family_contract.contract_digest()
            )
        }
        worth_server::WorthServerQuerySupportPosture::DirectStateSupported { family_contract } => {
            format!(
                "direct-state-supported:{}",
                family_contract.contract_digest()
            )
        }
        worth_server::WorthServerQuerySupportPosture::DirectInspectionSupported { family_contract } => {
            format!(
                "direct-inspection-supported:{}",
                family_contract.contract_digest()
            )
        }
        worth_server::WorthServerQuerySupportPosture::DirectProjectionSupported { family_contract } => {
            format!(
                "direct-projection-supported:{}",
                family_contract.contract_digest()
            )
        }
        worth_server::WorthServerQuerySupportPosture::DirectMutationSupported { family_contract } => {
            format!(
                "direct-mutation-supported:{}",
                family_contract.contract_digest()
            )
        }
        worth_server::WorthServerQuerySupportPosture::QueryMutationSupported { family_contract } => {
            format!(
                "query-mutation-supported:{}",
                family_contract.contract_digest()
            )
        }
        worth_server::WorthServerQuerySupportPosture::DownstreamDeliverySupported {
            family_contract,
            runtime_resume_support_posture,
            durable_resume_support_posture,
            contract_digest,
        } => format!(
            "downstream-delivery-supported:{}:{}:{}:{contract_digest}",
            family_contract.contract_digest(),
            runtime_resume_support_posture.as_str(),
            durable_resume_support_posture.as_str(),
        ),
        worth_server::WorthServerQuerySupportPosture::RuntimeBackedResumeSupported {
            family_contract,
            runtime_resume_support_posture,
            support_digest,
            contract_digest,
        } => format!(
            "runtime-backed-resume-supported:{}:{}:{support_digest}:{contract_digest}",
            family_contract.contract_digest(),
            runtime_resume_support_posture.as_str(),
        ),
        worth_server::WorthServerQuerySupportPosture::DurableResumeSupported {
            family_contract,
            durable_resume_support_posture,
            support_digest,
            contract_digest,
        } => format!(
            "durable-resume-supported:{}:{}:{support_digest}:{contract_digest}",
            family_contract.contract_digest(),
            durable_resume_support_posture.as_str(),
        ),
    }
}

pub(crate) fn projection_request() -> WorthServerDirectProjectionRequest {
    worth_server::WorthServerDirectProjectionRequest::new(
        "authorized-projection:test",
        "narrowed-result-shape:test",
        "policy:test",
        "tenant-schema:test",
        ["identity.id", "profile.display_name"],
    )
    .entity_identities()
    .view_local_identities()
    .display_field("profile.display_name")
}

pub(crate) fn projection_remask() -> WorthQueryRuntimeRemaskProjection {
    worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection::remasked(
        worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
        "policy:test",
        "tenant-truth:test",
        "tenant-schema:test",
        "relationship-proof:test",
        "schema-context:test",
    )
}

pub(crate) fn direct_read_success(
    outcome: worth_server::WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct read, got {other:?}"),
    }
}

pub(crate) fn direct_read_denied(
    outcome: worth_server::WorthServerDirectReadOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected denied direct read, got {other:?}"),
    }
}

pub(crate) fn direct_state_success(
    outcome: worth_server::WorthServerDirectStateOutcome,
) -> worth_server::WorthServerDirectState {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct state, got {other:?}"),
    }
}

pub(crate) fn direct_retained_posture_success(
    outcome: worth_proof::TransitionOutcome<
        worth_server::WorthServerDirectRetainedPosture,
        worth_server::WorthServerQueryHandoffDenial,
        worth_server::WorthServerQueryHandoffDeferred,
        worth_server::WorthServerQueryHandoffStale,
        worth_server::WorthServerQueryHandoffRebindRequired,
        worth_server::WorthServerQueryHandoffFailure,
    >,
) -> worth_server::WorthServerDirectRetainedPosture {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful retained posture, got {other:?}"),
    }
}

pub(crate) fn direct_projection_success(
    outcome: worth_server::WorthServerDirectProjectionOutcome,
) -> worth_server::WorthServerDirectProjection {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct projection, got {other:?}"),
    }
}

pub(crate) fn direct_delivery_success(
    outcome: worth_server::WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerDirectDeliveryContract {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct delivery contract, got {other:?}"),
    }
}

pub(crate) fn direct_delivery_denied(
    outcome: worth_server::WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected denied direct delivery contract, got {other:?}"),
    }
}

pub(crate) fn direct_lease_success(
    outcome: worth_server::WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerDirectLeaseDeclaration {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct lease declaration, got {other:?}"),
    }
}