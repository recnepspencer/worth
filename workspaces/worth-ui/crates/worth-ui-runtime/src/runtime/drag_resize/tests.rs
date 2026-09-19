#[test]
fn three_hundred_pointer_samples_publish_ten_previews_and_no_committed_truth() {
    let (mut runtime, root, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_durable_resize_catalog();
    let committed_before = runtime.committed_allocation_scope_count_for_test();
    let durable_before = runtime
        .durable_semantic_state()
        .expect("activated catalog owns durable semantic state");
    let mut publications = 0u16;
    let mut preview_projections = 0u16;
    for frame in 0..10u32 {
        let completion = runtime.execute_framework_turn(|turn| {
            turn.resize_preview(|source| {
                for sample in 0..30u32 {
                    let extent = crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(
                        (frame * 30 + sample) as f32,
                    )
                    .unwrap();
                    source
                        .admit_and_submit(crate::runtime::UiResizePreviewSample::new(root, extent))
                        .unwrap();
                }
            });
        });
        let (transition, _) = completion
            .into_pending_mounted_preview()
            .unwrap_or_else(|other| panic!("ordinary turn carries mounted preview: {other:?}"));
        let preview = transition.preview();
        let counters = preview.stream_counters();
        assert_eq!(preview.target(), root);
        assert_eq!(preview.candidate_count(), 1);
        assert!(preview.all_candidates_admitted());
        assert_eq!(
            preview.extent(),
            crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(
                (frame * 30 + 29) as f32,
            )
            .unwrap(),
        );
        let before = preview.capture_isolation_basis();
        let resolved = transition.finish(before);
        let crate::runtime::UiPreviewPaintIsolationOutcome::Verified(isolation) =
            resolved.isolation
        else {
            panic!("pointer-rate preview must preserve allocation truth")
        };
        assert_eq!(isolation.before(), isolation.after());
        assert_eq!(isolation.durable_mutations(), 0);
        assert_eq!(isolation.committed_receipts(), 0);
        preview_projections += 1;
        publications += counters.preview_publications();
        assert_eq!(counters.admitted_samples(), 30);
        assert_eq!(counters.durable_mutations(), 0);
        assert_eq!(counters.committed_receipts(), 0);
    }
    assert_eq!(publications, 10);
    assert_eq!(preview_projections, 10);
    assert_eq!(
        runtime.committed_allocation_scope_count_for_test(),
        committed_before
    );
    assert_eq!(runtime.durable_semantic_state().unwrap(), durable_before);
}

#[test]
fn bare_resize_preview_posture_is_rejected_at_admission() {
    let mut authority = crate::runtime::replacement::state_inventory::WorthUiTransientInteractionAdmissionAuthority::default();
    assert_eq!(
        authority.admit(
            crate::graph::UiGraphNodeIdentity::new(7),
            crate::runtime::WorthUiTransientInteractionState::ResizePreview,
        ),
        Err(crate::runtime::WorthUiTransientInteractionAdmissionDenial::ResizePreviewRequiresTypedSample),
    );
}

#[test]
fn preview_without_resize_planning_authority_denies_before_state_change() {
    let (mut runtime, root, _, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_production_catalog_activation();
    let committed_before = runtime.committed_allocation_scope_count_for_test();
    let completion = runtime.execute_framework_turn(|turn| {
        turn.resize_preview(|source| {
            source
                .admit_and_submit(crate::runtime::UiResizePreviewSample::new(
                    root,
                    crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(280.0).unwrap(),
                ))
                .expect("typed preview ingress admits independently of planning authority");
        });
    });
    assert!(matches!(
        completion.replan_transaction(),
        Some(
            crate::runtime::UiAllocationReplanTransactionOutcome::Denied(
                crate::runtime::UiAllocationReplanTransactionCommitDenial::ResizeBasisDenied
            )
        )
    ));
    assert_eq!(
        runtime.committed_allocation_scope_count_for_test(),
        committed_before
    );
}

#[test]
fn terminal_resize_mutates_once_and_commits_one_receipt_per_turn() {
    let (mut runtime, _, input) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_durable_resize_catalog();
    let extent = crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(320.0).unwrap();
    let first = runtime.execute_framework_turn(|turn| {
        turn.durable_resize(|source| {
            source
                .admit_and_submit(crate::runtime::UiDurableResizeCommitIntent::terminal(
                    input.clone(),
                    extent,
                ))
                .expect("terminal resize source admits");
        });
    });
    let replay_selection = first.replan_selection().unwrap().clone();
    let first = first
        .durable_resize_outcome()
        .unwrap_or_else(|| panic!("terminal resize commits through ordinary turn: {first:?}"));
    assert_eq!(first.counters().durable_mutations(), 1);
    assert_eq!(first.counters().committed_receipts(), 1);
    assert_eq!(first.committed_replan().receipts().len(), 1);
    let first_receipt = &first.committed_replan().receipts()[0];
    let first_digest = first_receipt.generation().planning_evidence_digest();
    assert_eq!(first_receipt.resize_basis().unwrap().extent(), extent);
    assert_eq!(
        first_receipt
            .equivalence_basis()
            .resize_basis()
            .unwrap()
            .extent(),
        extent,
    );
    assert_eq!(
        first
            .durable_semantic_state()
            .committed_resize(input.identity_digest())
            .unwrap()
            .extent(),
        extent,
    );
    let (replay, replay_state, replay_mutated) = runtime
        .replay_admitted_durable_transaction_for_test(
            &replay_selection,
            input.identity_digest(),
            extent,
        );
    assert!(matches!(
        replay,
        crate::runtime::UiAllocationReplanTransactionOutcome::Replayed(_)
    ));
    assert!(!replay_mutated);
    assert_eq!(
        replay_state
            .unwrap()
            .committed_resize(input.identity_digest())
            .unwrap()
            .extent(),
        extent,
    );
    let different_extent =
        crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(321.0).unwrap();
    let (not_replayed, unchanged_state, different_mutated) = runtime
        .replay_admitted_durable_transaction_for_test(
            &replay_selection,
            input.identity_digest(),
            different_extent,
        );
    assert!(matches!(
        not_replayed,
        crate::runtime::UiAllocationReplanTransactionOutcome::Denied(
            crate::runtime::UiAllocationReplanTransactionCommitDenial::AdmittedGenerationSetChanged
        )
    ));
    assert!(!different_mutated);
    assert_eq!(
        unchanged_state
            .unwrap()
            .committed_resize(input.identity_digest())
            .unwrap()
            .extent(),
        extent,
    );

    let repeated = runtime.execute_framework_turn(|turn| {
        turn.durable_resize(|source| {
            source
                .admit_and_submit(crate::runtime::UiDurableResizeCommitIntent::terminal(
                    input.clone(),
                    extent,
                ))
                .expect("equal terminal resize source admits");
        });
    });
    let repeated = repeated.durable_resize_outcome().unwrap_or_else(|| {
        panic!("equal terminal resize remains a typed committed outcome: {repeated:?}")
    });
    assert_eq!(repeated.counters().durable_mutations(), 0);
    assert_eq!(repeated.counters().committed_receipts(), 1);
    assert_eq!(
        repeated.committed_replan().receipts()[0]
            .generation()
            .planning_evidence_digest(),
        first_digest,
    );
    assert_eq!(runtime.committed_allocation_scope_count_for_test(), 2);

    let changed_extent =
        crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(360.0).unwrap();
    let changed = runtime.execute_framework_turn(|turn| {
        turn.durable_resize(|source| {
            source
                .admit_and_submit(crate::runtime::UiDurableResizeCommitIntent::terminal(
                    input.clone(),
                    changed_extent,
                ))
                .expect("changed terminal resize admits");
        });
    });
    let changed = changed
        .durable_resize_outcome()
        .expect("changed extent commits");
    assert_eq!(changed.counters().durable_mutations(), 1);
    assert_eq!(changed.counters().committed_receipts(), 1);
    let changed_receipt = &changed.committed_replan().receipts()[0];
    assert_eq!(
        changed_receipt.resize_basis().unwrap().extent(),
        changed_extent
    );
    assert_ne!(
        changed_receipt.generation().planning_evidence_digest(),
        first_digest,
    );
    assert_eq!(
        changed
            .durable_semantic_state()
            .committed_resize(
                changed_receipt
                    .resize_basis()
                    .unwrap()
                    .durable_identity_digest()
            )
            .unwrap()
            .extent(),
        changed_extent,
    );
}

#[test]
fn durable_resize_truth_change_does_not_counterfeit_mounted_projection_delta() {
    let (mut runtime, root, input) =
        crate::runtime::tests::production_catalog_activation_test_support::runtime_with_durable_resize_catalog();
    let predecessor_revision = runtime.allocation_truth_revision().revision();
    let predecessor_projection = runtime
        .allocation_receipt_ledger
        .mounted_projection_source(Some(predecessor_revision))
        .projection(root)
        .expect("activated allocation geometry is valid")
        .expect("durable target has a mounted allocation projection");
    let extent = crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(360.0).unwrap();

    let completion = runtime.execute_framework_turn(|turn| {
        turn.durable_resize(|source| {
            source
                .admit_and_submit(crate::runtime::UiDurableResizeCommitIntent::terminal(
                    input.clone(),
                    extent,
                ))
                .expect("activated durable source admits");
        });
    });
    assert!(completion.durable_resize_outcome().is_some());
    assert!(runtime.allocation_truth_revision().revision() > predecessor_revision);

    let projection_source = runtime
        .allocation_receipt_ledger
        .mounted_projection_source(Some(predecessor_revision));
    let crate::runtime::UiMountedAllocationProjectionDelta::Exact(delta) =
        projection_source.delta()
    else {
        panic!("retained predecessor must produce exact mounted allocation delta");
    };
    assert_eq!(delta.journal_entries_touched(), 1);
    let successor_projection = projection_source
        .projection(root)
        .expect("committed allocation geometry is valid")
        .expect("durable target has a committed mounted projection");
    assert_eq!(successor_projection, predecessor_projection);
}

#[test]
fn foreign_reconciliation_generation_denies_before_ingress_or_state_change() {
    let (mut runtime, _, active_input) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_durable_resize_catalog();
    let before = runtime.durable_semantic_state().unwrap();
    let (_, foreign_pending, _) = crate::runtime::tests::durable_resize_input_boundary_tests::splitter_pending_activation_with_provenance(0xBAD0_F00D);
    let foreign = foreign_pending
        .staged_replacement()
        .reconciliation_plan()
        .admitted_durable_resize_input("surface:main")
        .expect("foreign splitter input exists")
        .clone();
    assert_ne!(
        foreign.authority_generation(),
        active_input.authority_generation()
    );
    let mut denial = None;
    let completion = runtime.execute_framework_turn(|turn| {
        turn.durable_resize(|source| {
            denial = source
                .admit_and_submit(crate::runtime::UiDurableResizeCommitIntent::terminal(
                    foreign,
                    crate::runtime::UiResizeLogicalExtent::try_from_logical_pixels(400.0).unwrap(),
                ))
                .err();
        });
    });
    assert_eq!(
        denial,
        Some(crate::runtime::WorthUiDurableResizeSourceAdmissionDenial::ForeignReconciliationGeneration)
    );
    assert!(completion.into_execution().is_ok());
    assert_eq!(runtime.durable_semantic_state().unwrap(), before);
}
