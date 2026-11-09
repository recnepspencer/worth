use worth_foundational::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;

use crate::{config::WorthServerQueryHandoffConfig, WorthServerPreparedQueryHandoffKind};

use super::{
    WorthServerQueryHandoff, WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
    WorthServerQueryHandoffInput, WorthServerQueryHandoffOperation,
    WorthServerQueryWorkspaceBindingRequest,
};
use crate::{
    WorthServerOperationQuerySupportContext, WorthServerOperationReadinessDenialCode,
    WorthServerOperationReadinessFacade,
};

pub(crate) fn prepare_query_handoff(
    config: &WorthServerQueryHandoffConfig,
    input: WorthServerQueryHandoffInput,
) -> super::WorthServerQueryHandoffOutcome {
    let (operation_admission, operation) = input.into_parts();
    let admission = operation_admission.authorization_proof().admission();
    let diagnostics_profile = admission.request_context().diagnostics_profile();

    if let Some(denial) = validate_prepared_intent(admission, &operation) {
        return TransitionOutcome::Denied(denial);
    }

    let binding_request = WorthServerQueryWorkspaceBindingRequest::for_query_handoff(
        admission.resolved_request_context().clone(),
        operation.clone(),
    );
    let workspace = match config.workspace_provider().bind_workspace(&binding_request) {
        Ok(workspace) => workspace,
        Err(error) => {
            return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::WorkspaceBindingFailed,
                diagnostics_profile,
                format!("{}: {}", error.stage(), error.message()),
            ));
        }
    };

    let downstream_delivery_contract = match workspace.public_downstream_delivery_contract() {
        Ok(contract) => contract,
        Err(cause) => {
            return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause),
                diagnostics_profile,
                "downstream contract identity exceeded canonical digest bounds",
            ))
        }
    };
    let readiness = WorthServerOperationReadinessFacade::default();
    let operation_support_posture = match readiness.compose_support(
        &operation_admission,
        Some(WorthServerOperationQuerySupportContext::new(
            admission.query_handoff_intent().kind(),
            &operation,
            &workspace,
            &downstream_delivery_contract,
        )),
    ) {
        Ok(value) => value,
        Err(denial) => {
            return TransitionOutcome::Denied(map_readiness_denial(denial, diagnostics_profile));
        }
    };
    let support_posture = match operation_support_posture.query_support_posture() {
        Some(value) => value.clone(),
        None => {
            return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily,
                diagnostics_profile,
                "query handoff requires resolved query support posture before planning",
            ));
        }
    };
    let precondition_posture = readiness.default_precondition_posture(&operation_admission);
    let concurrency_class = match readiness.classify_concurrency(
        &operation_admission,
        &operation_support_posture,
        &precondition_posture,
    ) {
        Ok(value) => value,
        Err(denial) => {
            return TransitionOutcome::Denied(map_readiness_denial(denial, diagnostics_profile));
        }
    };

    let canonical_digest = canonical_digest(QueryHandoffDigestParts {
        tenant_id: admission
            .resolved_request_context()
            .request_context()
            .workspace_target()
            .tenant_id(),
        workspace_id: admission
            .resolved_request_context()
            .request_context()
            .workspace_target()
            .workspace_id(),
        workspace_name: workspace.name(),
        operation: &operation,
        operation_admission_digest: operation_admission.canonical_digest(),
        support_digest: operation_support_posture.canonical_digest(),
        precondition_digest: precondition_posture.canonical_digest(),
        concurrency_label: concurrency_class_label(&concurrency_class),
        contract_digest: downstream_delivery_contract.contract_for_reporting(),
    });

    TransitionOutcome::Success(WorthServerQueryHandoff::new(
        super::WorthServerQueryHandoffParts {
            operation_admission,
            operation,
            workspace,
            downstream_delivery_contract,
            operation_support_posture: operation_support_posture.clone(),
            support_composition_receipt: operation_support_posture.composition_receipt().clone(),
            precondition_posture,
            concurrency_class,
            support_posture,
            canonical_digest,
        },
    ))
}

fn validate_prepared_intent(
    admission: &crate::WorthServerAdmission,
    operation: &WorthServerQueryHandoffOperation,
) -> Option<WorthServerQueryHandoffDenial> {
    let prepared = admission.query_handoff_intent();
    match operation {
        WorthServerQueryHandoffOperation::QueryRead { operation_name }
            if prepared.kind() == WorthServerPreparedQueryHandoffKind::QueryRead
                && prepared.operation_name() == operation_name =>
        {
            None
        }
        WorthServerQueryHandoffOperation::QueryMutation { operation_name, .. }
            if prepared.kind() == WorthServerPreparedQueryHandoffKind::QueryMutation
                && prepared.operation_name() == operation_name =>
        {
            None
        }
        WorthServerQueryHandoffOperation::DirectRead { .. }
        | WorthServerQueryHandoffOperation::DirectState { .. }
        | WorthServerQueryHandoffOperation::DirectInspection { .. }
        | WorthServerQueryHandoffOperation::DirectProjection { .. }
        | WorthServerQueryHandoffOperation::DirectMutation { .. }
        | WorthServerQueryHandoffOperation::DownstreamDelivery { .. }
            if prepared.kind() == WorthServerPreparedQueryHandoffKind::WorthNativeSession =>
        {
            None
        }
        WorthServerQueryHandoffOperation::DirectRead { .. }
        | WorthServerQueryHandoffOperation::DirectState { .. }
        | WorthServerQueryHandoffOperation::DirectInspection { .. }
        | WorthServerQueryHandoffOperation::DirectProjection { .. }
            if prepared.kind() == WorthServerPreparedQueryHandoffKind::QueryRead =>
        {
            None
        }
        WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => None,
        _ => Some(WorthServerQueryHandoffDenial::new(
            WorthServerQueryHandoffDenialCode::PreparedIntentMismatch,
            admission.request_context().diagnostics_profile(),
            "query handoff operation does not match the middleware-admitted prepared intent",
        )),
    }
}

struct QueryHandoffDigestParts<'a> {
    tenant_id: &'a str,
    workspace_id: &'a str,
    workspace_name: &'a str,
    operation: &'a WorthServerQueryHandoffOperation,
    operation_admission_digest: &'a str,
    support_digest: &'a str,
    precondition_digest: &'a str,
    concurrency_label: &'a str,
    contract_digest: &'a str,
}

fn canonical_digest(parts: QueryHandoffDigestParts<'_>) -> String {
    let QueryHandoffDigestParts {
        tenant_id,
        workspace_id,
        workspace_name,
        operation,
        operation_admission_digest,
        support_digest,
        precondition_digest,
        concurrency_label,
        contract_digest,
    } = parts;
    format!(
        "worth-server-query-handoff-v3|tenant:{tenant_id}|workspace:{workspace_id}|bound:{workspace_name}|operation:{}|operation_admission:{operation_admission_digest}|support:{support_digest}|precondition:{precondition_digest}|concurrency:{concurrency_label}|contract:{contract_digest}",
        operation.canonical_label(),
    )
}

fn map_readiness_denial(
    denial: crate::WorthServerOperationReadinessDenial,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerQueryHandoffDenial {
    let code = match denial.code() {
        WorthServerOperationReadinessDenialCode::MissingQuerySupport
        | WorthServerOperationReadinessDenialCode::UnsupportedQuerySupport
        | WorthServerOperationReadinessDenialCode::UnsupportedProductSupport
        | WorthServerOperationReadinessDenialCode::UnknownProductSupport
        | WorthServerOperationReadinessDenialCode::FixtureOnlyProductSupport
        | WorthServerOperationReadinessDenialCode::IncompatibleSupportBasis => {
            WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
        }
        WorthServerOperationReadinessDenialCode::DownstreamDeliveryRequiresReadIntent => {
            WorthServerQueryHandoffDenialCode::DownstreamDeliveryRequiresReadIntent
        }
        WorthServerOperationReadinessDenialCode::RuntimeBackedResumeUnsupported => {
            WorthServerQueryHandoffDenialCode::RuntimeBackedResumeUnsupported
        }
        WorthServerOperationReadinessDenialCode::DurableResumeDeferred => {
            WorthServerQueryHandoffDenialCode::DurableResumeDeferred
        }
        WorthServerOperationReadinessDenialCode::InvalidPreconditionInput
        | WorthServerOperationReadinessDenialCode::PreconditionFailed => {
            WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
        }
    };
    WorthServerQueryHandoffDenial::new(code, diagnostics_profile, denial.detail())
}

fn concurrency_class_label(
    concurrency_class: &crate::WorthServerOperationConcurrencyClass,
) -> &'static str {
    match concurrency_class {
        crate::WorthServerOperationConcurrencyClass::ConcurrentSharedRead => {
            "concurrent-shared-read"
        }
        crate::WorthServerOperationConcurrencyClass::SerializeDeterministically => {
            "serialize-deterministically"
        }
    }
}
