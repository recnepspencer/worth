use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::intent_admission::certification_runtime;
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeSeamKey,
};
use crate::runtime::{WorthQueryIntentDeclaration, WorthQueryIntentInput};

use super::super::{RepresentativeArtifacts, WorthQueryLowerRuntimeRepresentativeEvidenceSource};

pub(crate) fn representative_runtime_intent_authority_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> RepresentativeArtifacts {
    let receipt = certification_intent_receipt(execution);
    route_planned_row(
        WorthQueryLowerRuntimeSeamKey::RuntimeIntentAuthorityAdapter,
        "Runtime intent authority seam",
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("strategy"),
                receipt.strategy_identity(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("outcome"),
                receipt.outcome_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("provenance"),
                receipt.execution_provenance_chain_digest(),
            )
            .seal(),
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("execution_binding"),
                receipt.execution_binding_digest(),
            )
            .seal(),
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("execution_provenance"),
                receipt.execution_provenance_chain_digest(),
            )
            .seal(),
    )
}

pub(crate) fn representative_intent_runtime_execution_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> RepresentativeArtifacts {
    let receipt = certification_intent_receipt(execution);
    route_planned_row(
        WorthQueryLowerRuntimeSeamKey::IntentRuntimeExecution,
        "Intent runtime execution",
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(WorthQueryEvidenceTag::new("intent"), receipt.intent_name())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit"),
                receipt.commit_evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("snapshot"),
                receipt.snapshot_evidence_identity(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("receipt"),
                receipt.receipt_digest(),
            )
            .seal(),
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("admission_decision"),
                receipt.admission_decision_digest(),
            )
            .seal(),
        receipt.receipt_identity().clone(),
    )
}

fn certification_intent_receipt(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> crate::runtime::WorthQueryIntentReceipt {
    let mut runtime = certification_runtime(execution);
    runtime
        .execute_intent(
            execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "lower-runtime-certification-intent",
                "Task",
                "v1",
                "lower-runtime-certification-intent-input",
                WorthQueryIntentInput::object([
                    ("collection", WorthQueryIntentInput::string("Task")),
                    (
                        "entity_identity",
                        WorthQueryIntentInput::string("intent-task-1"),
                    ),
                    ("title", WorthQueryIntentInput::string("Intent fixture")),
                ]),
            ),
        )
        .expect("intent runtime fixture should execute")
}

fn route_planned_row(
    seam_key: WorthQueryLowerRuntimeSeamKey,
    capability_label: &str,
    subject_identity: WorthQueryEvidenceIdentity,
    route_identity: WorthQueryEvidenceIdentity,
    retained_evidence_source: WorthQueryEvidenceIdentity,
) -> RepresentativeArtifacts {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        seam_key,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        capability_label,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "phase-six-intent-route-subject",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("subject"), &subject_identity)
        .seal(),
    );
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &retained_evidence_source,
    );
    let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility.clone(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "phase-six-intent-route",
            &route_identity,
        ),
    );
    let retained_evidence_identity =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "phase-six-intent-route",
            &retained_evidence_source,
        );
    let boundary_receipt = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
        &route_plan,
        &retained_evidence_identity,
    );
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        seam_key,
        &route_plan,
        &boundary_receipt,
        &retained_evidence_identity,
    );
    RepresentativeArtifacts {
        seam_key,
        request,
        eligibility,
        route_plan: Some(route_plan),
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    }
}
