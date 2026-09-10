use super::super::support::*;

#[test]
fn preview_discard_closeout_separates_temporary_writes_from_authoritative_residue() {
    let mut runtime = stateful_bridge_task_runtime();
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.preview-closeout",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");

    let outcome = {
        let mut preview = runtime
            .preview(test_session_label("discard closeout"))
            .expect("preview session should be admitted");
        preview.use_view(&live);
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("preview-temp-1")),
                    ("title.value", test_string_aspect_value("Temporary one")),
                ],
            ))
            .expect("first preview write should stage");
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("preview-temp-2")),
                    ("title.value", test_string_aspect_value("Temporary two")),
                ],
            ))
            .expect("second preview write should stage");
        preview.discard()
    };
    let closeout = outcome.closeout_evidence();

    assert!(outcome.discarded());
    assert_eq!(closeout.kind(), WorthQueryPreviewCloseoutKind::Discarded);
    assert_eq!(closeout.preview_binding_count(), 1);
    assert_eq!(closeout.live_binding_count(), 1);
    assert_eq!(closeout.preview_write_staging_count(), 2);
    assert_eq!(
        closeout.class_count(WorthQueryPreviewResidueClass::PreviewWriteStaging),
        2
    );
    assert_eq!(
        closeout.class_count(WorthQueryPreviewResidueClass::AuthoritativeResidue),
        0
    );
    assert_eq!(closeout.authoritative_residue_count(), 0);
    assert_eq!(closeout.effect_delivery_residue_count(), 0);
    assert_eq!(closeout.pending_write_intent_residue_count(), 0);
    assert_eq!(
        closeout.closeout_identity().as_str(),
        closeout.closeout_digest()
    );
    assert!(!closeout.closeout_digest().is_empty());
    assert!(runtime.read_live(&live).is_empty());
}

#[test]
fn preview_promotion_closeout_records_consumed_staging_without_preview_lane_mutation() {
    let mut runtime = stateful_bridge_task_runtime();
    runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare before preview-safe operation runs");
    let program = preview_safe_program();
    let installed = runtime
        .install_program(program)
        .expect("program should install");
    let operation = installed
        .operation("create_task")
        .expect("operation ref should build");
    let outcome = {
        let mut preview = runtime
            .preview_with_options(
                test_session_label("promotion closeout"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should be admitted");
        preview
            .run_operation(
                operation,
                vec![WorthQueryOperationInput::new(
                    "title",
                    WorthQueryProgramValue::string("Promoted closeout task"),
                )],
            )
            .expect("preview operation should stage");
        preview.promote().expect("preview promotion should succeed")
    };
    let closeout = outcome.closeout_evidence();

    assert!(outcome.promoted());
    assert_eq!(outcome.write_count(), 1);
    assert_eq!(closeout.kind(), WorthQueryPreviewCloseoutKind::Promoted);
    assert_eq!(closeout.preview_write_staging_count(), 1);
    assert_eq!(closeout.promoted_write_count(), 1);
    assert_eq!(
        closeout.class_count(WorthQueryPreviewResidueClass::PreviewWriteStaging),
        1
    );
    assert_eq!(closeout.authoritative_residue_count(), 0);
    assert_eq!(
        closeout.effect_policy(),
        WorthQueryEffectPolicy::SandboxedWriteIntent
    );
    assert_eq!(
        closeout.closeout_identity().as_str(),
        closeout.closeout_digest()
    );

    let view = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.after-promotion-closeout",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let rows = runtime.read_live(&view);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        test_native_string_value(&rows[0], "title.value").as_deref(),
        Some("Promoted closeout task")
    );
}

#[test]
fn preview_promotion_rejects_stale_basis_before_authority_execution() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(DriftingSnapshotIdentityAdapter::default())
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native preview promotion contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("drifting backend should build");

    let error = {
        let mut preview = runtime
            .preview(test_session_label("stale basis"))
            .expect("preview session should be admitted");
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("stale-preview")),
                    (
                        "title.value",
                        test_string_aspect_value("Should not promote"),
                    ),
                ],
            ))
            .expect("preview write should stage");
        preview
            .promote()
            .expect_err("drifting authoritative basis should deny promotion")
    };

    match error {
        WorthQueryRuntimeError::PreviewPromotionStaleBasis(evidence) => {
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::StaleBasis
            );
            assert_eq!(evidence.staged_preview_write_count(), 1);
            assert_eq!(evidence.promoted_write_count(), 0);
            assert_ne!(
                evidence.basis_snapshot_identity(),
                evidence.promotion_snapshot_identity()
            );
            assert!(!evidence.denial_digest().is_empty());
        }
        other => panic!("expected stale basis promotion denial, got {other:?}"),
    }
}

#[test]
fn preview_promotion_write_failure_is_typed_and_not_silently_dropped() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(DenyingWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native preview promotion contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("denying write backend should build");

    let error = {
        let mut preview = runtime
            .preview(test_session_label("write failure"))
            .expect("preview session should be admitted");
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("denied-preview")),
                    (
                        "title.value",
                        test_string_aspect_value("Denied preview write"),
                    ),
                ],
            ))
            .expect("preview write should stage");
        preview
            .promote()
            .expect_err("write authority denial should fail promotion")
    };

    match error {
        WorthQueryRuntimeError::PreviewPromotionWriteFailed { evidence } => {
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::WriteFailed
            );
            assert_eq!(evidence.staged_preview_write_count(), 1);
            assert_eq!(evidence.promoted_write_count(), 0);
            assert_eq!(evidence.failed_write_sequence(), Some(1));
            assert!(evidence.reason().contains("write authority denied"));
            assert!(!evidence.denial_digest().is_empty());
        }
        other => panic!("expected write failure promotion denial, got {other:?}"),
    }
}

#[test]
fn preview_promotion_rejects_multi_write_batch_before_partial_authority_execution() {
    let attempted_writes = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(CountingWriteAuthority {
            attempted_writes: attempted_writes.clone(),
        })
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native preview promotion contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("counting write backend should build");

    let error = {
        let mut preview = runtime
            .preview(test_session_label("multi write promotion"))
            .expect("preview session should be admitted");
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("preview-batch-1")),
                    (
                        "title.value",
                        test_string_aspect_value("First staged write"),
                    ),
                ],
            ))
            .expect("first preview write should stage");
        preview
            .write(insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("preview-batch-2")),
                    (
                        "title.value",
                        test_string_aspect_value("Second staged write"),
                    ),
                ],
            ))
            .expect("second preview write should stage");
        preview
            .promote()
            .expect_err("non-atomic multi-write promotion should deny before authority")
    };

    assert_eq!(attempted_writes.get(), 0);
    match error {
        WorthQueryRuntimeError::PreviewPromotionAtomicBatchUnsupported(evidence) => {
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::AtomicBatchUnsupported
            );
            assert_eq!(evidence.staged_preview_write_count(), 2);
            assert_eq!(evidence.promoted_write_count(), 0);
            assert_eq!(evidence.failed_write_sequence(), None);
            assert!(evidence.reason().contains("atomic promotion support"));
            assert!(!evidence.denial_digest().is_empty());
        }
        other => panic!("expected atomic batch promotion denial, got {other:?}"),
    }
}
