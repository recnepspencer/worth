use crate::declarative_live::{
    DeclarativeLiveQueryRequest, DeclarativeLiveViewShape, DeclarativeProjectionField,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::lower_runtime_routing::{
    LiveViewDeclarationAdmissionBoundaryReceipt, SignalInvalidationBoundaryReceipt,
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityRequest,
    WorthQueryLowerRuntimeReadmissionReceipt, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeSeamKey,
    WorthQueryLowerRuntimeSubjectIdentity, WriteAuthorityExecutionReceipt,
};
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQueryMutationDelta,
    WorthQueryMutationKind, WorthQueryMutationReceipt, WorthQuerySnapshotIdentity,
};
use crate::runtime::{
    build_bridge_authority_bundle, LiveViewDeclarationAdmissionReceipt,
    SignalInvalidationRoutingReceipt, WorthQueryAuthoredAspectMutation,
    WorthQueryBackendAdmissibleMutation, WorthQueryBasisAdmissionEvidenceRow,
    WorthQueryEffectPolicy, WorthQueryPreviewBasisAdmission, WorthQueryRuntimeEvidenceAuthority,
    WorthQueryRuntimeSourceAdapter, WorthQueryWriteCommand,
};
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};
use crate::session_label::WorthQuerySessionLabel;
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;

use super::{
    priority_value_touch, representative_bridge_authority_runtime, status_value_touch,
    RepresentativeArtifacts, RepresentativeSourceAdapter,
    WorthQueryLowerRuntimeRepresentativeEvidenceSource,
};

mod identity_fixtures;
mod inventory_fixtures;

use identity_fixtures::*;
pub(crate) use inventory_fixtures::*;

pub(crate) fn representative_live_view_schema_row(
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let request = DeclarativeLiveQueryRequest::new("Task", DeclarativeLiveViewShape::table())
        .project(DeclarativeProjectionField::from_authoring_parts(
            "title", "value",
        ))
        .project(DeclarativeProjectionField::from_authoring_parts(
            "status", "value",
        ));
    let admission_receipt =
        LiveViewDeclarationAdmissionReceipt::from_request("tasks.table", &request)?;
    let boundary_receipt = LiveViewDeclarationAdmissionBoundaryReceipt::from_request(
        "tasks.table",
        &request,
        admission_receipt,
    )?;
    Ok(RepresentativeArtifacts {
        seam_key: WorthQueryLowerRuntimeSeamKey::LiveViewSchemaAdmission,
        request: boundary_receipt
            .readmission_receipt()
            .eligibility()
            .request()
            .clone(),
        eligibility: boundary_receipt.readmission_receipt().eligibility().clone(),
        route_plan: None,
        boundary_receipt: boundary_receipt.boundary_execution_receipt().clone(),
        envelope: boundary_receipt.boundary_envelope().clone(),
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    })
}

pub(crate) fn representative_write_authority_row(
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let command = WorthQueryWriteCommand::Delete {
        entity_identity: representative_entity_identity("task-7")?,
    };
    let mutation_receipt = WorthQueryMutationReceipt::from_authoritative_parts(
        representative_commit_identity("commit-route-write-7")?,
        representative_snapshot_identity("snapshot-route-write-7")?,
        vec![WorthQueryMutationDelta::from_touched_aspects(
            "Task",
            representative_entity_identity("task-7")?,
            WorthQueryMutationKind::Deleted,
            vec![status_value_touch()],
        )?],
    );
    let execution = WriteAuthorityExecutionReceipt::from_command(&command, mutation_receipt)?;
    Ok(RepresentativeArtifacts {
        seam_key: WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
        request: execution.route_plan().eligibility().request().clone(),
        eligibility: execution.route_plan().eligibility().clone(),
        route_plan: Some(execution.route_plan().clone()),
        boundary_receipt: execution.boundary_execution_receipt().clone(),
        envelope: execution.boundary_envelope().clone(),
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    })
}

pub(crate) fn representative_signal_invalidation_row(
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let representative_task_identity = WorthQueryEntityIdentity::from_relational_record(
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 9, 0),
    );
    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: representative_task_identity.clone(),
        aspects: vec![
            WorthQueryAuthoredAspectMutation::new_set(
                status_value_touch(),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("ready"),
            )
            .expect("representative signal status aspect should build"),
            WorthQueryAuthoredAspectMutation::new_set(
                priority_value_touch(),
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("high"),
            )
            .expect("representative signal priority aspect should build"),
        ],
        metadata: Default::default(),
        naming_intent: None,
        continuity_intent: None,
    };
    let contracts =
        crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
            [status_value_touch(), priority_value_touch()]
                .map(representative_string_field_contract),
        )
        .expect("representative signal contracts should agree");
    let mutation = WorthQueryBackendAdmissibleMutation::from_authored_command(command, &contracts)
        .expect("representative signal mutation should satisfy native contracts");
    let bridge = representative_bridge_authority_runtime();
    let snapshot_identity = WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(1, 1),
    );
    let bridge_authority = build_bridge_authority_bundle(
        &bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            "Task",
            &representative_task_identity,
            WorthQueryMutationKind::Updated,
        ),
    )
    .expect("representative signal authority should build");
    let mutation_receipt = WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        WorthQueryCommitIdentity::from_relational_commit_id(1),
        snapshot_identity,
        vec![WorthQueryMutationDelta::from_touched_aspects(
            "Task",
            representative_task_identity,
            WorthQueryMutationKind::Updated,
            vec![status_value_touch(), priority_value_touch()],
        )?],
        bridge_authority,
    )
    .admit_runtime_write_authority();
    let routing = SignalInvalidationRoutingReceipt::from_mutation_receipt(&mutation_receipt)
        .expect("representative signal routing fixture must carry bridge authority");
    let boundary_receipt =
        SignalInvalidationBoundaryReceipt::from_mutation_receipt(&mutation_receipt, routing)?;
    Ok(RepresentativeArtifacts {
        seam_key: WorthQueryLowerRuntimeSeamKey::SignalInvalidationRouting,
        request: boundary_receipt
            .route_plan()
            .eligibility()
            .request()
            .clone(),
        eligibility: boundary_receipt.route_plan().eligibility().clone(),
        route_plan: Some(boundary_receipt.route_plan().clone()),
        boundary_receipt: boundary_receipt.boundary_execution_receipt().clone(),
        envelope: boundary_receipt.boundary_envelope().clone(),
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    })
}

fn representative_string_field_contract(
    touch: crate::runtime::WorthQueryAspectTouch,
) -> worth_foundational::facade::AspectContract {
    use worth_foundational::facade::{
        AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
        FieldDeclaration, FieldRequirement, ScalarAspectType, StructAspectShape,
    };

    let field = touch
        .native_field_path()
        .expect("representative field touch should contain a field")
        .fields()[0]
        .clone();
    let declaration = FieldDeclaration::new(
        field,
        ScalarAspectType::String,
        FieldRequirement::Optional,
        AbsenceLaw::Optional,
        AspectEvolutionPolicy::AdditiveFieldsAllowed,
    )
    .expect("representative field declaration should be coherent");
    AspectContract::struct_aspect(
        touch.native_aspect_key().clone(),
        AspectIdentity(1),
        AspectContractRevision(1),
        StructAspectShape::new([declaration]).expect("representative shape should be unique"),
    )
}

fn representative_commit_identity(
    label: impl AsRef<str>,
) -> Result<WorthQueryCommitIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryCommitIdentity::preview(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WriteReceiptCommitIdentity)
            .field_value(WorthQueryEvidenceTag::new("representative_commit"), label)
            .seal()?,
    ))
}

fn representative_snapshot_identity(
    label: impl AsRef<str>,
) -> Result<WorthQuerySnapshotIdentity, IdentityDerivationDenial> {
    Ok(WorthQuerySnapshotIdentity::preview(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity)
            .field_value(WorthQueryEvidenceTag::new("representative_snapshot"), label)
            .seal()?,
    ))
}

fn representative_entity_identity(
    label: impl AsRef<str>,
) -> Result<WorthQueryEntityIdentity, IdentityDerivationDenial> {
    Ok(crate::memory_workspace::admit_authored_entity_label(label)?)
}

pub(crate) fn representative_live_view_source_row(
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let request = DeclarativeLiveQueryRequest::new("Task", DeclarativeLiveViewShape::table())
        .project(DeclarativeProjectionField::from_authoring_parts(
            "identity", "id",
        ))
        .project(DeclarativeProjectionField::from_authoring_parts(
            "title", "value",
        ));
    let schema_view = QuerySchemaView::new(
        "certification-live-source",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
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
    );
    let mut source = RepresentativeSourceAdapter;
    let handle = source
        .declare_live_view(
            "tasks.certified".to_string(),
            request.clone(),
            schema_view.clone(),
        )
        .expect("live source declaration fixture should succeed");
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::LiveViewSourceDeclaration,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        "Live view source declaration",
        WorthQueryLowerRuntimeSubjectIdentity::compose("live-view-source-route-subject")
            .field_value(WorthQueryEvidenceTag::new("view"), handle.name())
            .field_value(WorthQueryEvidenceTag::new("target"), request.target())
            .field_shape(
                WorthQueryEvidenceTag::new("shape"),
                request.view_shape().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("schema_basis"),
                schema_view.basis().render_support_hex(),
            )
            .seal()?,
    )?;
    let eligibility = admitted_fixture_eligibility(
        request.clone(),
        "live-view-source-eligibility",
        handle.name(),
    )?;
    let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility.clone(),
        fixture_route_subject_identity("live-view-source-route", handle.name())?,
    );
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "representative-live-view-source",
            &fixture_retained_evidence_identity("representative-live-view-source", handle.name())?,
        )?;
    let boundary_receipt = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
        &route_plan,
        &retained_evidence,
    );
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        WorthQueryLowerRuntimeSeamKey::LiveViewSourceDeclaration,
        &route_plan,
        &boundary_receipt,
        &retained_evidence,
    )?;
    Ok(RepresentativeArtifacts {
        seam_key: WorthQueryLowerRuntimeSeamKey::LiveViewSourceDeclaration,
        request,
        eligibility,
        route_plan: Some(route_plan),
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    })
}

pub(crate) fn representative_preview_basis_row(
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let authority = WorthQueryRuntimeEvidenceAuthority::new();
    let admission = WorthQueryPreviewBasisAdmission::new(
        &authority,
        WorthQuerySessionLabel::scoped_strs("lower-runtime-routing", ["preview.tasks.certified"])
            .expect("fixture label should build"),
        WorthQueryEffectPolicy::DeriveOnly,
        WorthQueryBasisAdmissionEvidenceRow::rows_from_values([
            "preview-basis-evidence-a",
            "preview-basis-evidence-b",
        ])?,
    )?;
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        WorthQueryLowerRuntimeSeamKey::PreviewBasisAdmission,
        WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        "Preview basis admission",
        WorthQueryLowerRuntimeSubjectIdentity::compose("preview-basis-route-subject")
            .field_value(WorthQueryEvidenceTag::new("label"), admission.label())
            .field_shape(
                WorthQueryEvidenceTag::new("policy"),
                admission.effect_policy().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lane"),
                admission.authority_lane().as_str(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("evidence_count"),
                admission.evidence().len(),
            )
            .seal()?,
    )?;
    let admission_evidence_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_value(WorthQueryEvidenceTag::new("label"), admission.label())
            .field_shape(
                WorthQueryEvidenceTag::new("policy"),
                admission.effect_policy().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lane"),
                admission.authority_lane().as_str(),
            )
            .field_value_sequence(WorthQueryEvidenceTag::new("evidence"), admission.evidence())
            .seal()?;
    let eligibility =
        admitted_fixture_eligibility_from_evidence(request.clone(), &admission_evidence_identity)?;
    let retained_evidence =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "representative-preview-basis-admission",
            &fixture_retained_evidence_identity(
                "representative-preview-basis-admission",
                admission.authority_lane().as_str(),
            )?,
        )?;
    let handoff =
        WorthQueryLowerRuntimeReadmissionReceipt::new(eligibility.clone(), &retained_evidence);
    let boundary_receipt =
        WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_readmission_receipt(&handoff);
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_readmission_receipt(
        WorthQueryLowerRuntimeSeamKey::PreviewBasisAdmission,
        &handoff,
        &boundary_receipt,
    )?;
    Ok(RepresentativeArtifacts {
        seam_key: WorthQueryLowerRuntimeSeamKey::PreviewBasisAdmission,
        request,
        eligibility,
        route_plan: None,
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    })
}
