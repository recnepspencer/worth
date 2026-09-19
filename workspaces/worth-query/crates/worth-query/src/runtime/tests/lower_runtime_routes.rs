use super::support::*;

#[test]
fn runtime_live_view_denies_when_schema_boundary_receipt_drifts_from_request() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(DriftingSchemaReceiptAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(TestWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native lower-runtime route contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("backend should build with drifting schema boundary receipt");

    let error = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "external.tasks",
            task_live_request(),
            task_schema(),
        )
        .expect_err("drifted live admission boundary must deny installation");

    assert_live_installation_error(error, "backend-live-admission-receipt");
}

#[test]
fn runtime_write_denies_when_write_authority_route_receipt_drifts_from_command() {
    struct DriftingWriteAuthority;

    impl WorthQueryRuntimeWriteAuthorityAdapter for DriftingWriteAuthority {
        fn write(
            &mut self,
            bridge: &RuntimeBridge,
            relational_runtime: Option<&mut RelationalRuntime>,
            mutation: WorthQueryBackendAdmissibleMutation,
        ) -> Result<WriteAuthorityExecutionReceipt, WorthQueryWorkspaceError> {
            let mut authority = TestWriteAuthority;
            let honest = authority.write(bridge, relational_runtime, mutation)?;
            Ok(WriteAuthorityExecutionReceipt::from_command(
                &WorthQueryWriteCommand::Delete {
                    entity_identity: crate::memory_workspace::admit_authored_entity_label(
                        "drifted-entity",
                    ),
                },
                honest.mutation_receipt().clone(),
            ))
        }
    }

    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(DriftingWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native lower-runtime route contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("backend should build with drifting write route receipt");

    let error = runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("")),
                ("title.value", test_string_aspect_value("x")),
            ],
        ))
        .expect_err("drifted write route receipt must deny the write");

    assert_workspace_write_error(
        error,
        "lower-runtime capability request subject identity drifted",
    );
}

fn assert_live_installation_error(error: WorthQueryRuntimeError, expected_stage: &str) {
    match error {
        WorthQueryRuntimeError::LiveSubscriptionInstallation { stage, .. } => {
            assert_eq!(stage, expected_stage);
        }
        other => panic!("expected live installation denial, got {other:?}"),
    }
}

fn assert_workspace_write_error(error: WorthQueryRuntimeError, expected_message: &str) {
    match error {
        WorthQueryRuntimeError::Workspace(error) => {
            assert_eq!(error.to_string(), expected_message);
        }
        other => panic!("expected workspace denial, got {other:?}"),
    }
}
