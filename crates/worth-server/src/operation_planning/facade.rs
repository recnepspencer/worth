use crate::{
    config::WorthServerQueryHandoffConfig, WorthServerOperationConcurrencyClass,
    WorthServerOperationQuerySupportContext, WorthServerOperationReadinessClosure,
    WorthServerOperationReadinessFacade, WorthServerOperationRegistry,
    WorthServerPreparedQueryHandoffKind, WorthServerQueryHandoff, WorthServerQueryHandoffOperation,
    WorthServerQueryWorkspaceBindingRequest,
};

use super::{
    WorthServerLoweredOperationPlan, WorthServerOperationExecutionStrategy,
    WorthServerOperationPlanCounters, WorthServerOperationPlanDenial,
    WorthServerOperationPlanDenialCode, WorthServerOperationPlanEvidencePolicy,
    WorthServerOperationPlanReceipt, WorthServerOperationPlannerInput,
};

#[derive(Clone, Debug)]
pub struct WorthServerOperationPlanner {
    query_handoff_config: WorthServerQueryHandoffConfig,
    operation_registry: Option<WorthServerOperationRegistry>,
}

impl WorthServerOperationPlanner {
    pub(crate) fn with_operation_registry(
        query_handoff_config: WorthServerQueryHandoffConfig,
        operation_registry: WorthServerOperationRegistry,
    ) -> Self {
        Self {
            query_handoff_config,
            operation_registry: Some(operation_registry),
        }
    }

    pub fn lower(
        &self,
        input: WorthServerOperationPlannerInput,
    ) -> Result<WorthServerLoweredOperationPlan, WorthServerOperationPlanDenial> {
        let (operation_admission, operation, precondition_posture, bound_workspace) =
            input.into_parts();
        let admission = operation_admission.authorization_proof().admission();
        let diagnostics_profile = admission.request_context().diagnostics_profile();

        validate_prepared_intent(admission, &operation).map_err(|detail| {
            WorthServerOperationPlanDenial::new(
                WorthServerOperationPlanDenialCode::PreparedIntentMismatch,
                diagnostics_profile,
                detail,
            )
        })?;

        let workspace = match bound_workspace {
            Some(bound_workspace) => bound_workspace,
            None => {
                let binding_request = WorthServerQueryWorkspaceBindingRequest::for_query_handoff(
                    admission.resolved_request_context().clone(),
                    operation.clone(),
                );
                self.query_handoff_config
                    .workspace_provider()
                    .bind_workspace(&binding_request)
                    .map_err(|error| {
                        WorthServerOperationPlanDenial::new(
                            WorthServerOperationPlanDenialCode::WorkspaceBindingFailed,
                            diagnostics_profile,
                            format!("{}: {}", error.stage(), error.message()),
                        )
                    })?
            }
        };
        let downstream_delivery_contract = workspace
            .public_downstream_delivery_contract()
            .map_err(|cause| {
                WorthServerOperationPlanDenial::new(
                    WorthServerOperationPlanDenialCode::EvidenceIdentityDerivationDenied(cause),
                    diagnostics_profile,
                    "downstream contract identity exceeded canonical digest bounds",
                )
            })?;
        let readiness = match &self.operation_registry {
            Some(operation_registry) => {
                WorthServerOperationReadinessFacade::with_operation_registry(
                    operation_registry.clone(),
                )
            }
            None => WorthServerOperationReadinessFacade::default(),
        };
        let query_context = WorthServerOperationQuerySupportContext::new(
            admission.query_handoff_intent().kind(),
            &operation,
            &workspace,
            &downstream_delivery_contract,
        );
        let readiness_closure = readiness
            .close_readiness(
                &operation_admission,
                Some(query_context),
                precondition_posture,
            )
            .map_err(|denial| {
                WorthServerOperationPlanDenial::from_readiness_denial(denial, diagnostics_profile)
            })?;
        let query_handoff = build_query_handoff(
            operation_admission,
            operation,
            workspace,
            downstream_delivery_contract,
            readiness_closure,
        );
        let strategy = WorthServerOperationExecutionStrategy::from_authority_kind(
            query_handoff
                .operation_admission()
                .authority_footprint()
                .authority_kind(),
        );
        let evidence_policy =
            WorthServerOperationPlanEvidencePolicy::from_diagnostics_profile(diagnostics_profile);
        let counters = WorthServerOperationPlanCounters::new(
            support_rows_consulted(query_handoff.support_composition_receipt()),
            footprint_breadth(query_handoff.operation_admission().authority_footprint()),
            strategy,
            evidence_policy.materialization_lane(),
        );
        let plan_identity = canonical_plan_identity(&query_handoff, strategy);
        let expected_scheduler_lane = scheduler_lane(query_handoff.operation_admission());
        let receipt =
            WorthServerOperationPlanReceipt::new(super::WorthServerOperationPlanReceiptParts {
                support_composition_digest: query_handoff
                    .support_composition_receipt()
                    .canonical_digest()
                    .to_string(),
                footprint_digest: query_handoff
                    .operation_admission()
                    .authority_footprint()
                    .canonical_digest()
                    .to_string(),
                strategy,
                authorization_proof_digest: query_handoff
                    .operation_admission()
                    .authorization_proof()
                    .canonical_digest()
                    .to_string(),
                precondition_posture_digest: query_handoff
                    .precondition_posture()
                    .canonical_digest()
                    .to_string(),
                expected_scheduler_lane: expected_scheduler_lane.clone(),
                plan_identity,
                evidence_identity: evidence_policy.evidence_identity().to_string(),
            });
        Ok(WorthServerLoweredOperationPlan::new(
            query_handoff,
            strategy,
            evidence_policy,
            counters,
            receipt,
        ))
    }
}

fn build_query_handoff(
    operation_admission: crate::WorthServerOperationAdmissionPosture,
    operation: WorthServerQueryHandoffOperation,
    workspace: worth_query::facade::runtime::WorthQueryWorkspace,
    downstream_delivery_contract: worth_query::facade::runtime::WorthQueryRuntimeDownstreamDeliveryContract,
    readiness_closure: WorthServerOperationReadinessClosure,
) -> WorthServerQueryHandoff {
    let canonical_digest = format!(
        "worth-server-query-handoff-v3|tenant:{}|workspace:{}|bound:{}|operation:{}|operation_admission:{}|support:{}|precondition:{}|concurrency:{}|contract:{}",
        operation_admission
            .operation_request()
            .resolved_request_context()
            .request_context()
            .workspace_target()
            .tenant_id(),
        operation_admission
            .operation_request()
            .resolved_request_context()
            .request_context()
            .workspace_target()
            .workspace_id(),
        workspace.name(),
        operation.canonical_label(),
        operation_admission.canonical_digest(),
        readiness_closure.support_posture().canonical_digest(),
        readiness_closure.precondition_posture().canonical_digest(),
        concurrency_label(readiness_closure.concurrency_class()),
        downstream_delivery_contract.contract_for_reporting(),
    );
    let support_posture = readiness_closure
        .support_posture()
        .query_support_posture()
        .cloned()
        .unwrap_or_else(
            || crate::WorthServerQuerySupportPosture::ProductIndependent {
                label: "not-required".to_string(),
            },
        );
    WorthServerQueryHandoff::new(crate::query_handoff::WorthServerQueryHandoffParts {
        operation_admission,
        operation,
        workspace,
        downstream_delivery_contract,
        operation_support_posture: readiness_closure.support_posture().clone(),
        support_composition_receipt: readiness_closure
            .support_posture()
            .composition_receipt()
            .clone(),
        precondition_posture: readiness_closure.precondition_posture().clone(),
        concurrency_class: readiness_closure.concurrency_class(),
        support_posture,
        canonical_digest,
    })
}

fn validate_prepared_intent(
    admission: &crate::WorthServerAdmission,
    operation: &WorthServerQueryHandoffOperation,
) -> Result<(), &'static str> {
    let prepared = admission.query_handoff_intent();
    let admitted = match operation {
        WorthServerQueryHandoffOperation::QueryRead { operation_name } => {
            prepared.kind() == WorthServerPreparedQueryHandoffKind::QueryRead
                && prepared.operation_name() == operation_name
        }
        WorthServerQueryHandoffOperation::QueryMutation { operation_name, .. } => {
            prepared.kind() == WorthServerPreparedQueryHandoffKind::QueryMutation
                && prepared.operation_name() == operation_name
        }
        WorthServerQueryHandoffOperation::DirectRead { .. }
        | WorthServerQueryHandoffOperation::DirectState { .. }
        | WorthServerQueryHandoffOperation::DirectInspection { .. }
        | WorthServerQueryHandoffOperation::DirectProjection { .. }
            if prepared.kind() == WorthServerPreparedQueryHandoffKind::QueryRead =>
        {
            true
        }
        WorthServerQueryHandoffOperation::DirectRead { .. }
        | WorthServerQueryHandoffOperation::DirectState { .. }
        | WorthServerQueryHandoffOperation::DirectInspection { .. }
        | WorthServerQueryHandoffOperation::DirectProjection { .. }
        | WorthServerQueryHandoffOperation::DirectMutation { .. }
        | WorthServerQueryHandoffOperation::DownstreamDelivery { .. }
            if prepared.kind() == WorthServerPreparedQueryHandoffKind::WorthNativeSession =>
        {
            true
        }
        WorthServerQueryHandoffOperation::DownstreamDelivery { .. } => true,
        _ => false,
    };
    if admitted {
        Ok(())
    } else {
        Err("query handoff operation does not match the middleware-admitted prepared intent")
    }
}

fn support_rows_consulted(receipt: &crate::WorthServerOperationSupportCompositionReceipt) -> usize {
    receipt.query_rows_consulted().len() + receipt.product_rows_consulted().len()
}

fn footprint_breadth(footprint: &crate::WorthServerOperationAuthorityFootprint) -> usize {
    footprint.scope().breadth()
}

fn canonical_plan_identity(
    query_handoff: &WorthServerQueryHandoff,
    strategy: WorthServerOperationExecutionStrategy,
) -> String {
    format!(
        "worth-server-operation-plan-identity-v1|request={}|footprint={}|authorization={}|support={}|precondition={}|strategy={}|scheduler_lane={}",
        query_handoff
            .operation_admission()
            .operation_request()
            .identity()
            .canonical_digest(),
        query_handoff
            .operation_admission()
            .authority_footprint()
            .canonical_digest(),
        query_handoff
            .operation_admission()
            .authorization_proof()
            .canonical_digest(),
        query_handoff.operation_support_posture().canonical_digest(),
        query_handoff.precondition_posture().canonical_digest(),
        strategy.as_str(),
        scheduler_lane(query_handoff.operation_admission()),
    )
}

fn scheduler_lane(operation_admission: &crate::WorthServerOperationAdmissionPosture) -> String {
    match operation_admission.authority_footprint().authority_kind() {
        crate::WorthServerOperationAuthorityKind::SharedReadOnly
        | crate::WorthServerOperationAuthorityKind::DiagnosticsOnly => "shared-read".to_string(),
        crate::WorthServerOperationAuthorityKind::DeterministicSubmission => operation_admission
            .authority_metadata()
            .submission_lane()
            .expect("deterministic submission authority must carry a submission lane")
            .to_string(),
        crate::WorthServerOperationAuthorityKind::ProductDraftMutation => {
            let (product_session_identity, draft_scope) = operation_admission
                .authority_metadata()
                .product_draft_scope()
                .expect("product draft mutation authority must carry session and draft scope");
            format!("product-draft:{product_session_identity}:{draft_scope}")
        }
        crate::WorthServerOperationAuthorityKind::DurableProductMutation => {
            operation_admission
                .authority_metadata()
                .durable_product_mutation_scope()
                .expect("durable product mutation authority must carry scope and preconditions");
            format!(
                "durable-product:{}",
                operation_admission
                    .authority_footprint()
                    .scope()
                    .canonical_digest()
            )
        }
        crate::WorthServerOperationAuthorityKind::ProductSessionCoordination => {
            let (target, coordination_lane) = operation_admission
                .authority_metadata()
                .product_session_coordination_target()
                .expect("product session coordination authority must carry coordination metadata");
            match target {
                crate::WorthServerProductSessionCoordinationTarget::SessionCreation => format!(
                    "product-session-create:{}:{coordination_lane}",
                    operation_admission
                        .authority_footprint()
                        .scope()
                        .canonical_digest()
                ),
                crate::WorthServerProductSessionCoordinationTarget::ExistingSession {
                    product_session_identity,
                } => format!("product-session:{product_session_identity}:{coordination_lane}"),
            }
        }
        crate::WorthServerOperationAuthorityKind::BinaryStreaming
        | crate::WorthServerOperationAuthorityKind::LeaseCoordination => {
            "serialize-deterministically".to_string()
        }
    }
}

fn concurrency_label(concurrency_class: WorthServerOperationConcurrencyClass) -> &'static str {
    match concurrency_class {
        WorthServerOperationConcurrencyClass::ConcurrentSharedRead => "shared-read",
        WorthServerOperationConcurrencyClass::SerializeDeterministically => {
            "serialize-deterministically"
        }
    }
}
