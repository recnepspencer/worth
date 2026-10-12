use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::frontier_signal_adapter::SignalFrontierSurfaceEvidence;
use crate::intent_admission::{certification_bridge, certification_runtime};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeSeamKey,
};
use crate::runtime::{
    CausalInspection, QueryObservationReceipt, WorthQueryAspectMutationBuilder,
    WorthQueryReadFamily, WorthQueryReadResult, WorthQueryWorkspace,
};
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};
use worth_signal::facade::adapters::{
    InvalidationPerformedCounter, InvalidationPlanningEstimate, SignalInvalidationRealizedCounters,
};

use super::super::{
    title_value_touch, RepresentativeArtifacts, WorthQueryLowerRuntimeRepresentativeEvidenceSource,
};

pub(crate) fn representative_causal_bridge_materialization_row(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> RepresentativeArtifacts {
    let read_result = certification_read_result(execution);
    let inspection_basis = crate::basis_lifecycle::basis_lifecycle()
        .historical_snapshot("certification-read", true)
        .inspect()
        .expect("causal bridge inspection basis should admit");
    let observation =
        QueryObservationReceipt::from_read_receipt(read_result.receipt(), inspection_basis.clone());
    let plan = CausalInspection::for_observation(observation, inspection_basis)
        .why_replayed()
        .materialized_detail()
        .include_all_retained_evidence()
        .plan()
        .expect("causal bridge fixture should plan inspection");
    let artifact = plan
        .materialize_with_bridge(&certification_bridge())
        .expect("causal bridge fixture should materialize inspection");
    let evidence =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("artifact"),
                artifact.artifact_for_reporting(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("bridge_envelope"),
                artifact.bridge_envelope_for_reporting().unwrap_or("none"),
            )
            .seal();
    route_planned_row(
        WorthQueryLowerRuntimeSeamKey::CausalBridgeMaterialization,
        WorthQueryLowerRuntimeAuthorityOwner::RuntimeBridge,
        "Causal bridge materialization",
        evidence,
    )
}

pub(crate) fn representative_frontier_evidence_row() -> RepresentativeArtifacts {
    let planned = SignalFrontierSurfaceEvidence::from_planning_estimate(
        &InvalidationPlanningEstimate::default(),
    );
    let mut values = [0_u64; 24];
    values[InvalidationPerformedCounter::ReverseIndexCandidatesReturned as usize] = 1;
    values[InvalidationPerformedCounter::WorkItemsAdmitted as usize] = 1;
    values[InvalidationPerformedCounter::NodesEvaluated as usize] = 1;
    let executed = SignalFrontierSurfaceEvidence::from_realized_counters(
        &SignalInvalidationRealizedCounters::from_values(values),
    );
    let evidence =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(
                WorthQueryEvidenceTag::new("planned"),
                planned.surface_digest().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("executed"),
                executed.surface_digest().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("predicted_breadth"),
                planned.predicted_breadth().to_string(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("realized_breadth"),
                executed.realized_breadth().unwrap_or_default().to_string(),
            )
            .seal();
    route_planned_row(
        WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake,
        WorthQueryLowerRuntimeAuthorityOwner::Signal,
        "Frontier evidence intake",
        evidence,
    )
}

fn route_planned_row(
    seam_key: WorthQueryLowerRuntimeSeamKey,
    owner: WorthQueryLowerRuntimeAuthorityOwner,
    capability_label: &str,
    evidence: WorthQueryEvidenceIdentity,
) -> RepresentativeArtifacts {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        seam_key,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        owner,
        capability_label,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "phase-six-causal-signal-route-subject",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), &evidence)
        .seal(),
    );
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &evidence,
    );
    let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility.clone(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "phase-six-causal-signal-route",
            &evidence,
        ),
    );
    let retained_evidence_identity =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "phase-six-causal-signal-route",
            &evidence,
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

fn certification_read_result(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryReadResult {
    let mut workspace = certification_runtime(execution)
        .workspace("lower-runtime-causal-bridge")
        .expect("causal bridge workspace should build");
    workspace
        .insert(
            execution,
            "Task",
            |task: WorthQueryAspectMutationBuilder| {
                task.set_aspect(
                    title_value_touch(),
                    crate::runtime::WorthQueryAuthoredAspectValue::string("Causal fixture"),
                )
            },
        )
        .expect("causal bridge seed write should succeed");
    let family = certification_read_family(&mut workspace, "lower-runtime-causal-family");
    workspace
        .execute_read_family(execution, &family)
        .expect("causal bridge read should execute")
}

fn certification_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.local_detail(
                "user",
                QuerySchemaView::new(
                    "lower-runtime-causal-read",
                    [
                        SchemaFieldView::new(
                            crate::authoring::AspectName::new("identity")
                                .expect("schema aspect literal must be valid"),
                            crate::authoring::FieldName::new("id")
                                .expect("schema field literal must be valid"),
                            ScalarAspectType::String,
                        ),
                        SchemaFieldView::new(
                            crate::authoring::AspectName::new("title")
                                .expect("schema aspect literal must be valid"),
                            crate::authoring::FieldName::new("value")
                                .expect("schema field literal must be valid"),
                            ScalarAspectType::String,
                        ),
                    ],
                    [],
                ),
                |query| {
                    query.project(
                        AspectFieldSelector::new("identity", "id")
                            .expect("identity projection should build"),
                    )
                },
                |shape| {
                    shape.field(
                        AuthoredResultShapeField::new("identity", "id", "id")
                            .expect("identity result-shape field should build"),
                    )
                },
            )
        })
        .expect("causal bridge read family should define")
}
