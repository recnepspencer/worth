use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeReadmissionReceipt,
    WorthQueryLowerRuntimeRouteKind, WorthQueryLowerRuntimeSeamKey,
};
use crate::projection_consumption::ProjectionConsumptionSource;
use worth_runtime_bridge::facade::{
    materialize_bridge_grouped_truth_view_from_projection, materialize_bridge_row_set,
    AdmittedSourceRegistry, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeTruthViewSelector, RelationalBridgeRecordIdentityParts, SnapshotReadPacket,
    SnapshotReadPacketResult, SourceDeclaration, SourceDeclarationIdentity, TruthBranchIdentity,
    TruthSnapshotIdentity,
};
use worth_runtime_bridge::facade::{
    materialize_relational_authoritative_row_set, project_relational_grouped_truth,
};

use super::{
    aspect_key, bridge_grouped_projection_evidence, certification_query_write_receipt,
    grouped_projection_contract, projection_snapshot_identity, projection_source_evidence_identity,
    read_record, relational_grouped_projection_evidence, string_read, BridgeProjection,
    BridgeProjectionMember, RepresentativeArtifacts,
    WorthQueryLowerRuntimeRepresentativeEvidenceSource,
};

pub(crate) fn representative_projection_query_receipts_row() -> RepresentativeArtifacts {
    let receipt = certification_query_write_receipt();
    let source = ProjectionConsumptionSource::from_write_receipt(&receipt);

    readmission_projection_row(
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromQueryReceipts,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        "Projection source intake from Query receipts",
        projection_source_evidence_identity(&source, "query-receipt-source"),
        projection_source_evidence_identity(&source, "query-receipt-retained"),
    )
}

pub(crate) fn representative_projection_relational_row() -> RepresentativeArtifacts {
    let entity_one = RelationalBridgeRecordIdentityParts::entity(1, 1, 1);
    let entity_two = RelationalBridgeRecordIdentityParts::entity(1, 2, 1);
    let packet = SnapshotReadPacket::new(vec![
        string_read(entity_one, "identity.id"),
        string_read(entity_one, "status.lane"),
        string_read(entity_two, "identity.id"),
        string_read(entity_two, "status.lane"),
    ]);
    let result = SnapshotReadPacketResult::new(
        TruthSnapshotIdentity::from_relational_snapshot(
            worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts::new(6, 1),
        ),
        vec![
            read_record(
                &packet,
                0,
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("task-1"),
            ),
            read_record(
                &packet,
                1,
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("todo"),
            ),
            read_record(
                &packet,
                2,
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("task-2"),
            ),
            read_record(
                &packet,
                3,
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("doing"),
            ),
        ],
    );
    let row_set = materialize_relational_authoritative_row_set(&packet, &result)
        .expect("relational projection fixture should materialize row set");
    let grouped = project_relational_grouped_truth(
        &row_set,
        grouped_projection_contract("status", "identity.id", "status.lane"),
    )
    .expect("relational projection fixture should group row set");
    let source = ProjectionConsumptionSource::from_relational_grouped_projection(&grouped);

    readmission_projection_row(
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromRelationalArtifacts,
        WorthQueryLowerRuntimeAuthorityOwner::Relational,
        "Projection source intake from relational artifacts",
        relational_grouped_projection_evidence(&source, grouped.digest().as_str(), "source"),
        relational_grouped_projection_evidence(&source, grouped.digest().as_str(), "retained"),
    )
}

pub(crate) fn representative_projection_bridge_row(
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> RepresentativeArtifacts {
    let bridge = super::super::projection_bridge_runtime::projection_bridge_runtime();
    let declaration = SourceDeclaration::new(
        SourceDeclarationIdentity::from_stable_name("source:lower-runtime-certification"),
        BridgeTruthViewSelector::branch_snapshot(
            TruthBranchIdentity::from_relational_branch_id("main"),
            projection_snapshot_identity(),
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ]),
    );
    let registry = AdmittedSourceRegistry::freeze(vec![declaration.clone()])
        .expect("bridge source declaration should admit");
    let contract = registry
        .contract_for_declaration(&declaration)
        .expect("bridge source contract should exist");
    let entity_one = RelationalBridgeRecordIdentityParts::entity(1, 1, 1);
    let entity_two = RelationalBridgeRecordIdentityParts::entity(1, 2, 1);
    let packet = SnapshotReadPacket::new(vec![
        string_read(entity_one, "identity.id"),
        string_read(entity_one, "status"),
        string_read(entity_two, "identity.id"),
        string_read(entity_two, "status"),
    ]);
    let materialized = bridge
        .materialize_source_packet_batch(contract, vec![packet], resource_request)
        .expect("bridge projection fixture should materialize source packets");
    let row_set = materialize_bridge_row_set(materialized.first(), resource_request)
        .expect("bridge projection fixture should materialize row set");
    let grouped = materialize_bridge_grouped_truth_view_from_projection(
        &row_set,
        &BridgeProjection {
            snapshot_identity: projection_snapshot_identity(),
            grouping_aspect: aspect_key("status"),
            identity_binding_aspect_key: aspect_key("identity.id"),
            grouping_binding_aspect_key: aspect_key("status"),
            members: vec![
                BridgeProjectionMember::new(entity_one, "task-1", "todo"),
                BridgeProjectionMember::new(entity_two, "task-2", "doing"),
            ],
        },
    )
    .expect("bridge projection fixture should group truth view");
    let source = ProjectionConsumptionSource::from_bridge_grouped_truth_view(&grouped);

    readmission_projection_row(
        WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromBridgeArtifacts,
        WorthQueryLowerRuntimeAuthorityOwner::RuntimeBridge,
        "Projection source intake from bridge artifacts",
        bridge_grouped_projection_evidence(
            &source,
            &grouped.digest().bridge_admission_evidence(),
            "source",
        ),
        bridge_grouped_projection_evidence(
            &source,
            &grouped.digest().bridge_admission_evidence(),
            "retained",
        ),
    )
}

fn readmission_projection_row(
    seam_key: WorthQueryLowerRuntimeSeamKey,
    owner: WorthQueryLowerRuntimeAuthorityOwner,
    capability_label: &str,
    subject_identity: WorthQueryEvidenceIdentity,
    retained_evidence_source: WorthQueryEvidenceIdentity,
) -> RepresentativeArtifacts {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        seam_key,
        WorthQueryLowerRuntimeRouteKind::ReadmissionHandoff,
        owner,
        capability_label,
        crate::lower_runtime_routing::WorthQueryLowerRuntimeSubjectIdentity::compose(
            "phase-six-projection-route-subject",
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), &subject_identity)
        .seal(),
    );
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &retained_evidence_source,
    );
    let retained_evidence_identity =
        crate::lower_runtime_routing::worth_query_lower_runtime_retained_evidence_identity(
            "phase-six-projection-readmission",
            &retained_evidence_source,
        );
    let handoff = WorthQueryLowerRuntimeReadmissionReceipt::new(
        eligibility.clone(),
        &retained_evidence_identity,
    );
    let boundary_receipt =
        WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_readmission_receipt(&handoff);
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_readmission_receipt(
        seam_key,
        &handoff,
        &boundary_receipt,
    );
    RepresentativeArtifacts {
        seam_key,
        request,
        eligibility,
        route_plan: None,
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    }
}
