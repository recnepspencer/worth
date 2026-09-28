use crate::view_shape::ViewShapePlanArtifact;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, ScalarAspectType};
use worth_runtime_bridge::facade::{
    encode_snapshot_aspect_read_value, materialize_relational_authoritative_row_set,
    project_relational_grouped_truth, RelationalGroupedProjectionContract,
};
use worth_runtime_bridge::facade::{
    materialize_bridge_grouped_truth_view_from_projection, materialize_bridge_row_set,
    AspectKeySelector, BridgeMappingId, BridgeMappingRegistration, BridgeRuntimePolicy,
    BridgeTruthViewSelector, CoarseRoutingMode, MappingSelector,
    RelationalBridgeRecordIdentityParts, RuntimeBridge, RuntimeBridgeBuilder,
    SignalInvalidationScope, SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult,
    SnapshotReadRecord, SnapshotReadRequest, SourceDeclaration, SourceDeclarationIdentity,
    TruthBranchIdentity, TruthPatchScope, TruthPatchTargetSelector,
};

use super::super::LiveViewShapeExecutionEnvelope;
use super::grouped_truth_world::{
    grouped_member_record_identity, grouped_row, grouped_snapshot_identity, GroupedRowFixture,
    StaticSink, StaticSource, StaticSourceAdapter,
};

fn grouped_rows_packet(rows: &[GroupedRowFixture]) -> SnapshotReadPacket {
    SnapshotReadPacket::new(
        rows.iter()
            .enumerate()
            .flat_map(|(index, _row)| {
                let record_identity = grouped_member_record_identity(index);
                [
                    relational_snapshot_read(record_identity, "identity.id"),
                    relational_snapshot_read(record_identity, "profile.display_name"),
                    relational_snapshot_read(record_identity, "status.lane"),
                ]
            })
            .collect(),
    )
}

fn grouped_rows_result(
    rows: &[GroupedRowFixture],
    packet: &SnapshotReadPacket,
) -> SnapshotReadPacketResult {
    SnapshotReadPacketResult::new(
        grouped_snapshot_identity(),
        packet
            .reads()
            .iter()
            .map(|read| {
                let value = rows
                    .iter()
                    .enumerate()
                    .find_map(|(index, row)| {
                        (read.relational_record_identity_parts()
                            == Some(grouped_member_record_identity(index)))
                        .then(|| row.value_for_snapshot_read(read.aspect_key().as_str()))
                    })
                    .unwrap_or_else(|| {
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "unknown",
                        )
                    });
                SnapshotReadRecord::for_request(read, aspect_value(value))
            })
            .collect(),
    )
}

fn runtime(rows: &[GroupedRowFixture]) -> RuntimeBridge {
    let rows = std::sync::Arc::new(rows.to_vec());
    RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(StaticSource { rows: rows.clone() })
        .with_source_adapter(StaticSourceAdapter { rows: rows.clone() })
        .with_truth_branch_head_source(StaticSource { rows })
        .with_signal_sink(StaticSink)
        .register_source(registered_source())
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("mapping"),
            TruthPatchScope::new(
                MappingSelector::exact("result:task-1"),
                AspectKeySelector::exact(aspect_key("status")),
                TruthPatchTargetSelector::entity_field(field_key("lane")),
            ),
            SnapshotReadContract::scalar(aspect_key("status"), ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("signal:board"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("runtime should build for grouped truth-view tests")
}

fn registered_source() -> SourceDeclaration {
    SourceDeclaration::new(
        SourceDeclarationIdentity::from_stable_name("source:grouped-board"),
        BridgeTruthViewSelector::branch_snapshot(
            TruthBranchIdentity::from_bridge_harness_label("analysis"),
            grouped_snapshot_identity(),
        ),
        worth_runtime_bridge::facade::BridgeSourceCapabilitySet::new(vec![
            worth_runtime_bridge::facade::BridgeSourceCapability::SnapshotRead,
            worth_runtime_bridge::facade::BridgeSourceCapability::BranchRead,
        ]),
    )
}

pub(super) fn grouped_truth_view(
    plan: &ViewShapePlanArtifact,
) -> worth_runtime_bridge::facade::BridgeGroupedTruthViewArtifact {
    grouped_truth_view_with_rows(
        plan,
        &[
            grouped_row("task-1", "Ada", "todo"),
            grouped_row("task-2", "Bea", "doing"),
        ],
        "identity.id",
        None,
    )
}

pub(super) fn grouped_truth_view_with_rows(
    plan: &ViewShapePlanArtifact,
    rows: &[GroupedRowFixture],
    identity_field: &str,
    grouping_field_override: Option<&str>,
) -> worth_runtime_bridge::facade::BridgeGroupedTruthViewArtifact {
    let runtime = runtime(rows);
    let contract = runtime
        .admit_source(registered_source())
        .expect("registered source should admit");
    let packet = grouped_rows_packet(rows);
    let observation = runtime
        .materialize_source_packet(&contract, packet.clone())
        .expect("grouped source packet should materialize");
    let row_set = materialize_bridge_row_set(&observation).expect("row set");
    let relational_result = grouped_rows_result(rows, &packet);
    let relational_row_set =
        materialize_relational_authoritative_row_set(&packet, &relational_result)
            .expect("relational row set");
    let grouping_field = match plan
        .grouped_planning_artifact()
        .expect("grouped plan should carry grouped planning")
        .native_grouping_aspect_key()
        .as_str()
    {
        "status" => "status.lane",
        "profile" => "profile.display_name",
        other => other,
    };
    let relational_projection = project_relational_grouped_truth(
        &relational_row_set,
        relational_grouped_projection_contract(
            plan.grouped_planning_artifact()
                .expect("grouped plan should carry grouped planning")
                .native_grouping_aspect_key()
                .as_str(),
            identity_field,
            grouping_field_override.unwrap_or(grouping_field),
        ),
    )
    .expect("relational grouped projection");

    materialize_bridge_grouped_truth_view_from_projection(&row_set, &relational_projection)
        .expect("grouped truth view")
}

pub(super) fn assert_grouped_delta_counters_are_debt_free(
    execution: &LiveViewShapeExecutionEnvelope,
    expected_transition_count: usize,
    expected_lane_count: usize,
) {
    assert_eq!(
        execution.counters().grouped_delta_row_count(),
        expected_transition_count
    );
    assert_eq!(
        execution.counters().grouped_membership_transition_count(),
        expected_transition_count
    );
    assert_eq!(
        execution.counters().grouped_lane_count(),
        expected_lane_count
    );
    assert_eq!(
        execution.counters().view_family_refresh_admission_count(),
        0
    );
    assert_eq!(execution.counters().complexity_status_debt_count(), 0);
}

fn aspect_value(value: AspectValue) -> AspectValue {
    encode_snapshot_aspect_read_value(&value)
}

fn relational_snapshot_read(
    record_identity: RelationalBridgeRecordIdentityParts,
    aspect: &str,
) -> SnapshotReadRequest {
    SnapshotReadRequest::for_relational_record(
        record_identity,
        SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String),
    )
}

fn relational_grouped_projection_contract(
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

pub(super) fn aspect_key(label: &str) -> AspectKey {
    AspectKey::new(label).expect("test grouped projection aspect key must be foundational")
}

fn field_key(label: &str) -> FieldKey {
    FieldKey::new(label.to_owned()).expect("test grouped projection field key must be foundational")
}
