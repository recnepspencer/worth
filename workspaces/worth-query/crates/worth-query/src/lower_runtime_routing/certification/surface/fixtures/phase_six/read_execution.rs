use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::facade::foundation::{
    basis_lifecycle, preflight_execution_basis, resolve_snapshot_basis, BasisAuthorityFamily,
    BasisResolutionMode, ExecutionBasisIntent, ResolvedSnapshotIdentity, SnapshotLineageClass,
};
use crate::facade::policy::{
    admit_query_basis_context, QueryContextBindingSource, ScopedQueryBasisContext,
};
use crate::intent_admission::certification_runtime;
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeSeamKey,
};
use crate::runtime::{WorthQueryReadFamily, WorthQueryReadResult, WorthQueryWorkspace};
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};

use super::super::{RepresentativeArtifacts, WorthQueryLowerRuntimeRepresentativeEvidenceSource};

pub(crate) fn representative_compose_read_row() -> RepresentativeArtifacts {
    let mut workspace = certification_workspace("lower-runtime-compose-read");
    let result = workspace
        .compose_read(read_declaration())
        .expect("compose-read fixture should execute");

    route_planned_read_row(
        WorthQueryLowerRuntimeSeamKey::ComposeRead,
        "Composed current read",
        &result,
    )
}

pub(crate) fn representative_execute_read_family_row() -> RepresentativeArtifacts {
    let mut workspace = certification_workspace("lower-runtime-execute-read-family");
    let family = certification_read_family(&mut workspace, "lower-runtime-read-family");
    let result = workspace
        .execute_read_family(&family)
        .expect("execute-read-family fixture should execute");

    route_planned_read_row(
        WorthQueryLowerRuntimeSeamKey::ExecuteReadFamily,
        "Defined read-family execution",
        &result,
    )
}

pub(crate) fn representative_execute_read_family_in_basis_context_row() -> RepresentativeArtifacts {
    let mut workspace = certification_workspace("lower-runtime-execute-read-family-basis");
    let family = certification_read_family(&mut workspace, "lower-runtime-basis-family");
    let snapshot_identity = workspace
        .snapshot_identity()
        .expect("the representative fixture requires an open workspace");
    let snapshot_evidence_identity = snapshot_identity.evidence_identity();
    let context = branch_context_for_family(
        &family,
        snapshot_evidence_identity.terminal_projection_for_reporting(),
    );
    let result = workspace
        .execute_read_family_in_basis_context(&family, &context)
        .expect("execute-read-family-in-basis-context fixture should execute");

    route_planned_read_row(
        WorthQueryLowerRuntimeSeamKey::ExecuteReadFamilyInBasisContext,
        "Basis-context read-family execution",
        &result,
    )
}

pub(crate) fn representative_runtime_current_read_graph_row() -> RepresentativeArtifacts {
    let mut workspace = certification_workspace("lower-runtime-runtime-current-read");
    let family = certification_read_family(&mut workspace, "lower-runtime-current-family");
    let result = workspace
        .execute_read_family(&family)
        .expect("runtime current-read fixture should execute");

    route_planned_read_row(
        WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeCurrentReadGraph,
        "Current read execution",
        &result,
    )
}

pub(crate) fn representative_runtime_basis_context_read_graph_row() -> RepresentativeArtifacts {
    let mut workspace = certification_workspace("lower-runtime-runtime-basis-read");
    let family = certification_read_family(&mut workspace, "lower-runtime-runtime-basis-family");
    let snapshot_identity = workspace
        .snapshot_identity()
        .expect("the representative fixture requires an open workspace");
    let snapshot_evidence_identity = snapshot_identity.evidence_identity();
    let context = branch_context_for_family(
        &family,
        snapshot_evidence_identity.terminal_projection_for_reporting(),
    );
    let result = workspace
        .execute_read_family_in_basis_context(&family, &context)
        .expect("runtime basis-context read fixture should execute");

    route_planned_read_row(
        WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeBasisContextReadGraph,
        "Basis-context read execution",
        &result,
    )
}

fn route_planned_read_row(
    seam_key: WorthQueryLowerRuntimeSeamKey,
    capability_label: &str,
    result: &WorthQueryReadResult,
) -> RepresentativeArtifacts {
    let read_evidence = read_result_evidence_identity(result);
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        seam_key,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::RuntimeBridge,
        capability_label,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "phase-six-read-execution-route-subject",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("read"), &read_evidence)
        .seal(),
    );
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &read_evidence,
    );
    let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility.clone(),
        crate::lower_runtime_routing::WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            "read-execution-engine-route",
            &read_evidence,
        ),
    );
    let retained_evidence_identity =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "phase-six-read-execution-route",
            &read_evidence,
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

fn read_result_evidence_identity(result: &WorthQueryReadResult) -> WorthQueryEvidenceIdentity {
    let receipt = result.receipt();
    let snapshot_evidence_identity = receipt.snapshot_evidence_identity();
    let mut builder =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("engine"),
                receipt.execution_engine().as_str(),
            )
            .field_value(WorthQueryEvidenceTag::new("query"), receipt.query_digest())
            .field_value(WorthQueryEvidenceTag::new("basis"), receipt.basis_digest())
            .field_value(
                WorthQueryEvidenceTag::new("result"),
                receipt.result_digest(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("snapshot"),
                &snapshot_evidence_identity,
            );
    if let Some(provenance) = receipt.execution_provenance_chain_digest() {
        builder = builder.field_value(WorthQueryEvidenceTag::new("provenance"), provenance);
    }
    builder.seal()
}

fn certification_workspace(label: &str) -> WorthQueryWorkspace {
    certification_runtime()
        .workspace(label)
        .expect("lower-runtime read fixture workspace should build")
}

fn certification_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, read_declaration())
        .expect("lower-runtime read fixture family should define")
}

fn read_declaration() -> impl FnOnce(
    crate::runtime::WorthQueryReadBuilder,
) -> Result<
    crate::runtime::WorthQueryReadGraph,
    crate::runtime::WorthQueryReadDenial,
> {
    |read| {
        read.local_detail(
            "user",
            QuerySchemaView::new(
                "lower-runtime-read-fixture",
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
    }
}

fn branch_context_for_family(
    family: &WorthQueryReadFamily,
    snapshot_token: &str,
) -> ScopedQueryBasisContext {
    let preflight = runtime_preflight_for_family(family, snapshot_token);
    admit_query_basis_context(
        basis_lifecycle().branch_head(snapshot_token, true),
        QueryContextBindingSource::RuntimeBranch(&preflight),
    )
    .expect("branch read fixture context should admit")
}

fn runtime_preflight_for_family(
    family: &WorthQueryReadFamily,
    snapshot_token: &str,
) -> crate::facade::foundation::ExecutionPreflightBundle {
    let intent = ExecutionBasisIntent::new(
        BasisAuthorityFamily::Runtime,
        SnapshotLineageClass::CurrentHead,
        false,
    );
    let identity = ResolvedSnapshotIdentity::new(
        BasisAuthorityFamily::Runtime,
        None,
        crate::evidence_identity::WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity,
        )
        .field_value(
            crate::evidence_identity::WorthQueryEvidenceTag::new("fixture_snapshot"),
            snapshot_token,
        )
        .seal(),
        family.read_graph().schema_basis().clone(),
        SnapshotLineageClass::CurrentHead,
    );
    let basis = resolve_snapshot_basis(intent, identity, BasisResolutionMode::RuntimeDirect)
        .expect("read fixture runtime basis should resolve");
    preflight_execution_basis(family.read_graph().execution_plan().clone(), basis)
        .expect("read fixture preflight should build")
}
