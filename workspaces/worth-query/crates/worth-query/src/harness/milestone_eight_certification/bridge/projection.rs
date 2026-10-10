use super::*;

pub(in crate::harness::milestone_eight_certification) fn grouped_rows_packet(
    rows: &[GroupedRowFixture],
) -> SnapshotReadPacket {
    SnapshotReadPacket::new(
        rows.iter()
            .flat_map(|row| {
                let record_parts = milestone_eight_record_parts(row.member_key());
                [
                    string_snapshot_read(record_parts, "identity.id"),
                    string_snapshot_read(record_parts, "profile.display_name"),
                    string_snapshot_read(record_parts, "status.lane"),
                ]
            })
            .collect(),
    )
}

pub(in crate::harness::milestone_eight_certification) fn grouped_rows_result(
    rows: &[GroupedRowFixture],
    packet: &SnapshotReadPacket,
) -> SnapshotReadPacketResult {
    SnapshotReadPacketResult::new(
        milestone_eight_snapshot_identity(),
        packet
            .reads()
            .iter()
            .map(|read| {
                let value = rows
                    .iter()
                    .find_map(|row| {
                        (read.relational_record_identity_parts()
                            == Some(milestone_eight_record_parts(row.member_key())))
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

pub(in crate::harness::milestone_eight_certification) fn grouped_runtime(
    rows: &[GroupedRowFixture],
) -> RuntimeBridge {
    let rows = std::sync::Arc::new(rows.to_vec());
    RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(StaticSource { rows: rows.clone() })
        .with_source_adapter(StaticSourceAdapter { rows: rows.clone() })
        .with_truth_branch_head_source(StaticSource { rows })
        .with_signal_sink(StaticSink)
        .register_source(grouped_registered_source())
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("mapping"),
            TruthPatchScope::new(
                MappingSelector::exact(
                    milestone_eight_record_parts("task-1").terminal_projection_for_reporting(),
                ),
                AspectKeySelector::exact(aspect_key("status")),
                TruthPatchTargetSelector::entity_field(field_key("lane")),
            ),
            SnapshotReadContract::scalar(aspect_key("status"), ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("signal:board"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("runtime should build for grouped certification")
}

pub(in crate::harness::milestone_eight_certification) fn grouped_registered_source(
) -> SourceDeclaration {
    SourceDeclaration::new(
        SourceDeclarationIdentity::from_stable_name("source:grouped-board"),
        BridgeTruthViewSelector::branch_snapshot(
            milestone_eight_branch_identity(),
            milestone_eight_snapshot_identity(),
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ]),
    )
}

pub(in crate::harness::milestone_eight_certification) fn grouped_truth_view_for_plan(
    plan: &crate::view_shape::ViewShapePlanArtifact,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeGroupedTruthViewArtifact {
    grouped_truth_view_for_plan_with_rows(
        plan,
        &[
            grouped_row("task-1", "Ada", "todo"),
            grouped_row("task-2", "Bea", "doing"),
        ],
        resource_request,
    )
}

pub(in crate::harness::milestone_eight_certification) fn grouped_truth_view_for_plan_with_rows(
    plan: &crate::view_shape::ViewShapePlanArtifact,
    rows: &[GroupedRowFixture],
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeGroupedTruthViewArtifact {
    let runtime = grouped_runtime(rows);
    let contract = runtime
        .admit_source(grouped_registered_source())
        .expect("registered source should admit");
    let packet = grouped_rows_packet(rows);
    let observation = runtime
        .materialize_source_packet(&contract, packet.clone(), resource_request)
        .expect("grouped source packet should materialize");
    let row_set = materialize_bridge_row_set(&observation, resource_request).expect("row set");
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
            "identity.id",
            grouping_field,
        ),
    )
    .expect("relational grouped projection");

    materialize_bridge_grouped_truth_view_from_projection(&row_set, &relational_projection)
        .expect("grouped truth view")
}
