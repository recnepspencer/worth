use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport,
    WorthQueryRuntimeRemaskProjection, WorthQueryRuntimeRemaskReasonKind,
    WorthQueryRuntimeSupportProfile,
};
use worth_server::{
    request_context::DiagnosticRichnessProfile, WorthServer, WorthServerDirectProjectionRequest,
    WorthServerQuerySupportPosture, WorthServerRequestContext, WorthServerResponseEnvelope,
    WorthServerWorthNativeSession,
};

use {crate::shared_support_direct_context_runtime::RemaskWorkspaceProvider, crate::shared_support_worth_native_assertions::operator_evidence_record, crate::shared_support_worth_native_runtime::{
        build_server, build_server_with_profiled_workspace, build_server_with_workspace_provider,
        server_with_request_context_default, worth_native_session_input_builder,
    }};

use crate::certification_suite::certification_bundle::WorthServerCertificationBundle;

pub fn standard_server() -> WorthServer {
    build_server(true)
}

pub fn forensic_server() -> WorthServer {
    server_with_request_context_default(DiagnosticRichnessProfile::Forensic)
}

pub fn remask_server() -> WorthServer {
    build_server_with_workspace_provider(RemaskWorkspaceProvider::new(projection_remask()), true)
}

pub fn runtime_backed_server() -> WorthServer {
    build_server_with_profiled_workspace(WorthQueryRuntimeSupportProfile::scaffold_backend_profile())
}

pub fn durable_later_server() -> WorthServer {
    build_server_with_profiled_workspace(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::unsupported(
                WorthQueryRuntimeFacadeFamily::DurableArtifacts,
                "durable restart and artifact reload stay deferred for direct certification lanes",
            ),
        ),
    )
}

pub fn worth_native_session_for_branch(
    server: &WorthServer,
    branch_id: Option<&str>,
) -> WorthServerWorthNativeSession {
    worth_native_session_for_target(server, None, branch_id)
}

pub fn worth_native_session_for_target(
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
        TransitionOutcome::Success(session) => session,
        other => panic!("expected Worth-native certification session, got {other:?}"),
    }
}

pub fn direct_bundle(
    server: &WorthServer,
    request_context: &WorthServerRequestContext,
    response: WorthServerResponseEnvelope,
) -> WorthServerCertificationBundle {
    WorthServerCertificationBundle::from_response_and_evidence(
        request_context_digest(request_context),
        response.clone(),
        operator_evidence_record(server, response),
    )
}

pub fn request_context_digest(request_context: &WorthServerRequestContext) -> String {
    format!(
        "principal={};tenant={};workspace={};branch={};diagnostics={:?}",
        request_context.authenticated_principal().principal_id(),
        request_context.workspace_target().tenant_id(),
        request_context.workspace_target().workspace_id(),
        request_context.branch_target().canonical_label(),
        request_context.diagnostics_profile(),
    )
}

pub fn support_posture_digest(posture: &WorthServerQuerySupportPosture) -> String {
    match posture {
        WorthServerQuerySupportPosture::ProductIndependent { label } => {
            format!("product-independent:{label}")
        }
        WorthServerQuerySupportPosture::PrimaryGraphApplicationSupported { basis_token } => {
            format!("primary-graph-application-supported:{basis_token}")
        }
        WorthServerQuerySupportPosture::QueryReadSupported { family_contract } => {
            format!("query-read-supported:{}", family_contract.contract_digest())
        }
        WorthServerQuerySupportPosture::DirectReadSupported { family_contract } => {
            format!(
                "direct-read-supported:{}",
                family_contract.contract_digest()
            )
        }
        WorthServerQuerySupportPosture::DirectStateSupported { family_contract } => {
            format!(
                "direct-state-supported:{}",
                family_contract.contract_digest()
            )
        }
        WorthServerQuerySupportPosture::DirectInspectionSupported { family_contract } => {
            format!(
                "direct-inspection-supported:{}",
                family_contract.contract_digest()
            )
        }
        WorthServerQuerySupportPosture::DirectProjectionSupported { family_contract } => {
            format!(
                "direct-projection-supported:{}",
                family_contract.contract_digest()
            )
        }
        WorthServerQuerySupportPosture::DirectMutationSupported { family_contract } => {
            format!(
                "direct-mutation-supported:{}",
                family_contract.contract_digest()
            )
        }
        WorthServerQuerySupportPosture::QueryMutationSupported { family_contract } => {
            format!(
                "query-mutation-supported:{}",
                family_contract.contract_digest()
            )
        }
        WorthServerQuerySupportPosture::DownstreamDeliverySupported {
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
        WorthServerQuerySupportPosture::RuntimeBackedResumeSupported {
            family_contract,
            runtime_resume_support_posture,
            support_digest,
            contract_digest,
        } => format!(
            "runtime-backed-resume-supported:{}:{}:{support_digest}:{contract_digest}",
            family_contract.contract_digest(),
            runtime_resume_support_posture.as_str(),
        ),
        WorthServerQuerySupportPosture::DurableResumeSupported {
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

pub fn projection_request() -> WorthServerDirectProjectionRequest {
    WorthServerDirectProjectionRequest::new(
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

fn projection_remask() -> WorthQueryRuntimeRemaskProjection {
    WorthQueryRuntimeRemaskProjection::remasked(
        WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
        "policy:test",
        "tenant-truth:test",
        "tenant-schema:test",
        "relationship-proof:test",
        "schema-context:test",
    )
}

pub fn direct_read_success(
    outcome: worth_server::WorthServerDirectReadOutcome,
) -> worth_server::WorthServerDirectRead {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct read, got {other:?}"),
    }
}

pub fn direct_read_denied(
    outcome: worth_server::WorthServerDirectReadOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected denied direct read, got {other:?}"),
    }
}

pub fn direct_state_success(
    outcome: worth_server::WorthServerDirectStateOutcome,
) -> worth_server::WorthServerDirectState {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct state, got {other:?}"),
    }
}

pub fn direct_retained_posture_success(
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
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful retained posture, got {other:?}"),
    }
}

pub fn direct_projection_success(
    outcome: worth_server::WorthServerDirectProjectionOutcome,
) -> worth_server::WorthServerDirectProjection {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct projection, got {other:?}"),
    }
}

pub fn direct_delivery_success(
    outcome: worth_server::WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerDirectDeliveryContract {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct delivery contract, got {other:?}"),
    }
}

pub fn direct_delivery_denied(
    outcome: worth_server::WorthServerDirectDeliveryOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected denied direct delivery contract, got {other:?}"),
    }
}

pub fn direct_lease_success(
    outcome: worth_server::WorthServerDirectLeaseDeclarationOutcome,
) -> worth_server::WorthServerDirectLeaseDeclaration {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct lease declaration, got {other:?}"),
    }
}
