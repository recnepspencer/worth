use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath, RawAuthoredQuery,
    RawAuthoredResultShape, RootEntityKey,
};
use crate::authorized_projection::AuthorizedProjectionArtifact;
use crate::authorized_projection::{
    derive_authorized_projection, PolicyAspectMask, PolicyInfluenceSet,
};
use crate::canonicalization::CanonicalResultShapeArtifact;
use crate::projection_consumption::{
    ProjectMaterializedFacts, ProjectionConsumptionAuthoringSurface,
    ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError,
    ProjectionSourceFamily,
};
use crate::query_context::QueryContextExecutionArtifact;
use crate::runtime::{WorthQueryReadReceipt, WorthQueryWriteReceipt};
use worth_foundational::facade::{AspectKey, AspectValue, ScalarAspectType};
use worth_runtime_bridge::facade::{
    encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
    project_relational_grouped_truth, RelationalAuthoritativeRowSetArtifact,
    RelationalGroupedProjectionArtifact, RelationalGroupedProjectionContract,
};
use worth_runtime_bridge::facade::{
    BridgeGroupedTruthViewArtifact, BridgeMaterializedRowSetArtifact,
};
use worth_runtime_bridge::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    TruthSnapshotIdentity,
};
use worth_runtime_bridge::facade::{
    SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    SnapshotReadRequest,
};

fn projection_artifacts() -> (CanonicalResultShapeArtifact, AuthorizedProjectionArtifact) {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("task").unwrap())
        .project(AspectFieldSelector::new("identity", "id").unwrap())
        .project(AspectFieldSelector::new("status", "lane").unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .field(AuthoredResultShapeField::new("status", "lane", "lane").unwrap())
        .build()
        .unwrap();
    let canonical = GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap();
    let authorized_projection = derive_authorized_projection(
        canonical.query(),
        canonical.result_shape(),
        "policy:test",
        "schema:test",
        &PolicyAspectMask::allow_all(),
        &PolicyInfluenceSet::none(),
        8,
        8,
    )
    .unwrap();

    (canonical.result_shape().clone(), authorized_projection)
}

fn relational_row_set() -> RelationalAuthoritativeRowSetArtifact {
    let entity_one = relational_record_parts(1);
    let entity_two = relational_record_parts(2);
    let entity_one_identity = relational_snapshot_read(entity_one, "identity.id");
    let entity_one_lane = relational_snapshot_read(entity_one, "status.lane");
    let entity_two_identity = relational_snapshot_read(entity_two, "identity.id");
    let entity_two_lane = relational_snapshot_read(entity_two, "status.lane");
    let packet = SnapshotReadPacket::new(vec![
        entity_one_identity.clone(),
        entity_one_lane.clone(),
        entity_two_identity.clone(),
        entity_two_lane.clone(),
    ]);
    materialize_relational_authoritative_row_set(
        &packet,
        &SnapshotReadPacketResult::new(
            test_snapshot_identity(),
            vec![
                SnapshotReadRecord::for_request(
                    &entity_one_identity,
                    aspect_value(
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "task-1",
                        ),
                    ),
                ),
                SnapshotReadRecord::for_request(
                    &entity_one_lane,
                    aspect_value(
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "todo",
                        ),
                    ),
                ),
                SnapshotReadRecord::for_request(
                    &entity_two_identity,
                    aspect_value(
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "task-2",
                        ),
                    ),
                ),
                SnapshotReadRecord::for_request(
                    &entity_two_lane,
                    aspect_value(
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "doing",
                        ),
                    ),
                ),
            ],
        ),
    )
    .unwrap()
}

fn assert_declares(
    declaration: Result<ProjectionConsumptionDeclaration, ProjectionConsumptionDeclarationError>,
    expected_family: ProjectionSourceFamily,
    expected_shape: &str,
) {
    let declaration = declaration.expect("phase 1 declaration should build");
    assert_eq!(declaration.source().family(), expected_family);
    assert_eq!(declaration.binding().result_shape_digest(), expected_shape);
}

#[test]
fn query_owned_phase_one_entry_points_exist() {
    let _read_method: fn(
        &WorthQueryReadReceipt,
        &CanonicalResultShapeArtifact,
        &AuthorizedProjectionArtifact,
        ProjectMaterializedFacts,
    ) -> Result<
        ProjectionConsumptionDeclaration,
        ProjectionConsumptionDeclarationError,
    > = WorthQueryReadReceipt::declare_projection_fact_consumption;
    let _write_method: fn(
        &WorthQueryWriteReceipt,
        &str,
        &AuthorizedProjectionArtifact,
        ProjectMaterializedFacts,
    ) -> Result<
        ProjectionConsumptionDeclaration,
        ProjectionConsumptionDeclarationError,
    > = WorthQueryWriteReceipt::declare_projection_fact_consumption;
    let _query_context_method: fn(
        &QueryContextExecutionArtifact,
        &AuthorizedProjectionArtifact,
        ProjectMaterializedFacts,
    ) -> Result<
        ProjectionConsumptionDeclaration,
        ProjectionConsumptionDeclarationError,
    > = QueryContextExecutionArtifact::declare_projection_fact_consumption;
}

#[test]
fn source_locked_phase_one_authoring_surfaces_exist_for_all_declared_families() {
    let _relational_row_set_surface: fn(
        &RelationalAuthoritativeRowSetArtifact,
        &str,
        &AuthorizedProjectionArtifact,
    ) -> ProjectionConsumptionAuthoringSurface =
        ProjectionConsumptionAuthoringSurface::from_relational_row_set;
    let _relational_grouped_surface: fn(
        &RelationalGroupedProjectionArtifact,
        &str,
        &AuthorizedProjectionArtifact,
    ) -> ProjectionConsumptionAuthoringSurface =
        ProjectionConsumptionAuthoringSurface::from_relational_grouped_projection;
    let _bridge_row_set_surface: fn(
        &BridgeMaterializedRowSetArtifact,
        &str,
        &AuthorizedProjectionArtifact,
    ) -> ProjectionConsumptionAuthoringSurface =
        ProjectionConsumptionAuthoringSurface::from_bridge_truth_view_row_set;
    let _bridge_grouped_surface: fn(
        &BridgeGroupedTruthViewArtifact,
        &str,
        &AuthorizedProjectionArtifact,
    ) -> ProjectionConsumptionAuthoringSurface =
        ProjectionConsumptionAuthoringSurface::from_bridge_grouped_truth_view;
}

#[test]
fn relational_phase_one_authoring_surfaces_build_declarations() {
    let (result_shape, authorized_projection) = projection_artifacts();
    let row_set = relational_row_set();
    let grouped_projection = project_relational_grouped_truth(
        &row_set,
        grouped_projection_contract("status", "identity.id", "status.lane"),
    )
    .unwrap();

    assert_declares(
        ProjectMaterializedFacts::declare()
            .source(
                ProjectionConsumptionAuthoringSurface::from_relational_row_set(
                    &row_set,
                    result_shape.digest().as_str(),
                    &authorized_projection,
                ),
            )
            .entity_identities()
            .view_local_identities()
            .build(),
        ProjectionSourceFamily::RelationalRowSet,
        result_shape.digest().as_str(),
    );
    assert_declares(
        ProjectMaterializedFacts::declare()
            .source(
                ProjectionConsumptionAuthoringSurface::from_relational_grouped_projection(
                    &grouped_projection,
                    result_shape.digest().as_str(),
                    &authorized_projection,
                ),
            )
            .memberships()
            .relation_endpoints()
            .view_local_identities()
            .build(),
        ProjectionSourceFamily::RelationalGroupedProjection,
        result_shape.digest().as_str(),
    );
}

fn aspect_value(value: AspectValue) -> AspectValue {
    encode_snapshot_aspect_read_value(&value)
}

fn relational_snapshot_read(
    record_parts: RelationalBridgeRecordIdentityParts,
    aspect: &str,
) -> SnapshotReadRequest {
    SnapshotReadRequest::for_relational_record(
        record_parts,
        SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String),
    )
}

fn relational_record_parts(slot: u64) -> RelationalBridgeRecordIdentityParts {
    RelationalBridgeRecordIdentityParts::entity(1, slot, 1)
}

fn test_snapshot_identity() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        1, 1,
    ))
}

fn grouped_projection_contract(
    grouping_aspect: &str,
    identity_binding_aspect: &str,
    grouping_binding_aspect: &str,
) -> RelationalGroupedProjectionContract {
    RelationalGroupedProjectionContract::new(
        aspect_key(grouping_aspect),
        aspect_key(identity_binding_aspect),
        aspect_key(grouping_binding_aspect),
    )
}

fn aspect_key(label: &str) -> AspectKey {
    AspectKey::new(label).expect("test aspect key must be foundational")
}
