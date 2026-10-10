use worth_foundational::facade::AspectValue;

use crate::facade::BridgeGroupedBindingValueFamily;

use super::materialize_bridge_grouped_truth_view_from_projection;

#[path = "grouped_truth_view_tests/support.rs"]
mod support;

use support::{
    projection, projection_with_grouping, row_set, row_set_with_ambiguous_grouping_binding,
    row_set_with_struct_grouping_binding, row_set_with_struct_identity_binding, standard_members,
    TestProjectionMember,
};

#[test]
fn grouped_truth_view_preserves_row_and_lane_pairing() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let grouped = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            vec![
                TestProjectionMember {
                    row_identity: "entity-1".to_string(),
                    identity_value: AspectValue::String("task-1".into()),
                    grouping_value: AspectValue::String("todo".into()),
                },
                TestProjectionMember {
                    row_identity: "entity-2".to_string(),
                    identity_value: AspectValue::String("task-2".into()),
                    grouping_value: AspectValue::String("doing".into()),
                },
            ],
        ),
    )
    .expect("grouped truth view");

    let members = grouped.members();
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].row_identity().as_str(), "entity-1");
    assert_eq!(
        members[0].identity_value(),
        &AspectValue::String("task-1".into())
    );
    assert_eq!(
        members[0].lane().value(),
        &AspectValue::String("todo".into())
    );
    assert_eq!(members[0].lane().grouping_aspect(), "status");
    assert_eq!(
        members[0].lane().native_grouping_aspect_key().as_str(),
        "status"
    );
    assert_eq!(members[1].row_identity().as_str(), "entity-2");
    assert_eq!(
        members[1].identity_value(),
        &AspectValue::String("task-2".into())
    );
    assert_eq!(
        members[1].lane().value(),
        &AspectValue::String("doing".into())
    );
}

#[test]
fn grouped_truth_view_digest_is_derived_from_projection_contract_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let source_row_set = row_set(resource_request);
    let status_grouped = materialize_bridge_grouped_truth_view_from_projection(
        &source_row_set,
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            standard_members(),
        ),
    )
    .expect("status grouped view");
    let workflow_grouped = materialize_bridge_grouped_truth_view_from_projection(
        &source_row_set,
        &projection_with_grouping(
            "workflow-status",
            "snapshot-a",
            "identity.id",
            "status.lane",
            standard_members(),
        ),
    )
    .expect("workflow grouped view");

    assert_ne!(status_grouped.digest(), workflow_grouped.digest());
}

#[test]
fn grouped_truth_view_rejects_basis_snapshot_mismatch() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-b",
            "identity.id",
            "status.lane",
            standard_members(),
        ),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        super::BridgeGroupedTruthViewError::BasisSnapshotMismatch { .. }
    ));
}

#[test]
fn grouped_truth_view_rejects_projection_row_count_mismatch() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            vec![standard_members()[0].clone()],
        ),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        super::BridgeGroupedTruthViewError::RowCountMismatch { .. }
    ));
}

#[test]
fn grouped_truth_view_rejects_missing_projection_row() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            vec![
                TestProjectionMember {
                    row_identity: "entity-1".to_string(),
                    identity_value: AspectValue::String("task-1".into()),
                    grouping_value: AspectValue::String("todo".into()),
                },
                TestProjectionMember {
                    row_identity: "entity-404".to_string(),
                    identity_value: AspectValue::String("task-2".into()),
                    grouping_value: AspectValue::String("doing".into()),
                },
            ],
        ),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        super::BridgeGroupedTruthViewError::MissingProjectionRow { .. }
    ));
}

#[test]
fn grouped_truth_view_rejects_identity_and_grouping_parity_mismatches() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let identity_error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            vec![
                TestProjectionMember {
                    row_identity: "entity-1".to_string(),
                    identity_value: AspectValue::String("wrong-task".into()),
                    grouping_value: AspectValue::String("todo".into()),
                },
                TestProjectionMember {
                    row_identity: "entity-2".to_string(),
                    identity_value: AspectValue::String("task-2".into()),
                    grouping_value: AspectValue::String("doing".into()),
                },
            ],
        ),
    )
    .unwrap_err();
    assert!(matches!(
        identity_error,
        super::BridgeGroupedTruthViewError::IdentityParityMismatch { .. }
    ));

    let grouping_error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            vec![
                TestProjectionMember {
                    row_identity: "entity-1".to_string(),
                    identity_value: AspectValue::String("task-1".into()),
                    grouping_value: AspectValue::String("done".into()),
                },
                TestProjectionMember {
                    row_identity: "entity-2".to_string(),
                    identity_value: AspectValue::String("task-2".into()),
                    grouping_value: AspectValue::String("doing".into()),
                },
            ],
        ),
    )
    .unwrap_err();
    assert!(matches!(
        grouping_error,
        super::BridgeGroupedTruthViewError::GroupingParityMismatch { .. }
    ));
}

#[test]
fn grouped_truth_view_rejects_missing_identity_and_grouping_aspects() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let identity_error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-a",
            "identity.missing",
            "status.lane",
            standard_members(),
        ),
    )
    .unwrap_err();
    assert!(matches!(
        identity_error,
        super::BridgeGroupedTruthViewError::MissingIdentityAspect { .. }
    ));

    let grouping_error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.missing",
            standard_members(),
        ),
    )
    .unwrap_err();
    assert!(matches!(
        grouping_error,
        super::BridgeGroupedTruthViewError::MissingGroupingAspect { .. }
    ));
}

#[test]
fn grouped_truth_view_rejects_ambiguous_whole_aspect_grouping_binding() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set_with_ambiguous_grouping_binding(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            standard_members(),
        ),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        super::BridgeGroupedTruthViewError::AmbiguousGroupingAspect {
            row_identity,
            aspect_key,
            matching_projection_count: 2,
        } if row_identity == "entity-1" && aspect_key == "status.lane"
    ));
}

#[test]
fn grouped_truth_view_rejects_struct_identity_binding_before_member_materialization() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set_with_struct_identity_binding(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            standard_members(),
        ),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        super::BridgeGroupedTruthViewError::UnsupportedIdentityAspectValueFamily {
            row_identity,
            aspect_key,
            value_family: BridgeGroupedBindingValueFamily::Struct,
            validated_value_canonical_basis,
        } if row_identity == "entity-1"
            && aspect_key == "identity.id"
            && validated_value_canonical_basis.contains("identity.id")
    ));
}

#[test]
fn grouped_truth_view_rejects_struct_grouping_binding_before_member_materialization() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let error = materialize_bridge_grouped_truth_view_from_projection(
        &row_set_with_struct_grouping_binding(resource_request),
        &projection(
            "snapshot-a",
            "identity.id",
            "status.lane",
            standard_members(),
        ),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        super::BridgeGroupedTruthViewError::UnsupportedGroupingAspectValueFamily {
            row_identity,
            aspect_key,
            value_family: BridgeGroupedBindingValueFamily::Struct,
            validated_value_canonical_basis,
        } if row_identity == "entity-1"
            && aspect_key == "status.lane"
            && validated_value_canonical_basis.contains("status.lane")
    ));
}
