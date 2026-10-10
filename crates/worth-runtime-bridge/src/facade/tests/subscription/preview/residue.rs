use super::super::support::*;

#[test]
fn preview_discard_emits_zero_residue_proof_for_all_categories() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active) =
        preview_active_detail_subscription("discard-zero", resource_request);
    let preview_active_identity = preview_active
        .preview_active_subscription_identity()
        .clone();
    let residue_scope_identity = preview_active.preview_residue_scope_identity().clone();
    let work_trace = preview_work_trace(&runtime, &preview_active, "discard-zero");
    let residue_index = runtime.build_subscription_preview_residue_scope_index(
        &preview_active,
        work_trace.zero_residue_inputs(),
    );
    let residue_index_identity = residue_index.preview_residue_scope_index_identity().clone();

    let proof = runtime
        .prove_preview_scope_discard_residue(preview_active, residue_index)
        .expect("zero residue should prove preview discard");

    assert_eq!(
        proof.preview_active_subscription_identity(),
        &preview_active_identity
    );
    assert_eq!(
        proof.preview_residue_scope_index_identity(),
        &residue_index_identity
    );
    assert_eq!(
        proof.preview_residue_scope_identity(),
        &residue_scope_identity
    );
    assert_eq!(proof.artifact_records().len(), 7);
    assert_eq!(proof.total_residue_count(), 0);
    let expected_categories = [
        crate::facade::BridgeSubscriptionPreviewResidueCategory::AuthoritativeTruthSubscription,
        crate::facade::BridgeSubscriptionPreviewResidueCategory::BridgeSubscriptionRegistry,
        crate::facade::BridgeSubscriptionPreviewResidueCategory::ActiveDelivery,
        crate::facade::BridgeSubscriptionPreviewResidueCategory::FanoutConsumerContract,
        crate::facade::BridgeSubscriptionPreviewResidueCategory::Continuation,
        crate::facade::BridgeSubscriptionPreviewResidueCategory::CheckpointReplay,
        crate::facade::BridgeSubscriptionPreviewResidueCategory::SignalVisible,
    ];
    assert_eq!(
        proof
            .artifact_records()
            .iter()
            .map(|record| record.category())
            .collect::<Vec<_>>(),
        expected_categories
    );
    for record in proof.artifact_records() {
        assert_eq!(
            record.preview_residue_scope_identity(),
            &residue_scope_identity
        );
        assert_eq!(record.residue_count(), 0);
        assert_eq!(
            record.evidence_digest(),
            format!(
                "preview-work-zero-residue|trace={}|scope={}|record={}|category={}",
                work_trace.digest(),
                residue_scope_identity.as_str(),
                expected_work_record_digest_for_residue_category(&work_trace, record.category()),
                record.category().as_str(),
            ),
            "evidence digest must bind the residue category through preview work"
        );
    }
    assert_eq!(proof.counters().subscription_preview_discard_count(), 1);
    assert_eq!(
        proof.counters().subscription_preview_residue_check_count(),
        7
    );
    assert_eq!(
        proof
            .counters()
            .subscription_preview_non_scope_registry_scan_count(),
        0
    );
}

fn expected_work_record_digest_for_residue_category(
    work_trace: &crate::facade::BridgeSubscriptionPreviewWorkTrace,
    category: crate::facade::BridgeSubscriptionPreviewResidueCategory,
) -> &str {
    match category {
        crate::facade::BridgeSubscriptionPreviewResidueCategory::AuthoritativeTruthSubscription
        | crate::facade::BridgeSubscriptionPreviewResidueCategory::BridgeSubscriptionRegistry => {
            work_trace.record_digest_for(crate::facade::BridgeSubscriptionPreviewWorkKind::Routing)
        }
        crate::facade::BridgeSubscriptionPreviewResidueCategory::ActiveDelivery
        | crate::facade::BridgeSubscriptionPreviewResidueCategory::FanoutConsumerContract => {
            work_trace.record_digest_for(crate::facade::BridgeSubscriptionPreviewWorkKind::Delivery)
        }
        crate::facade::BridgeSubscriptionPreviewResidueCategory::Continuation
        | crate::facade::BridgeSubscriptionPreviewResidueCategory::CheckpointReplay => work_trace
            .record_digest_for(crate::facade::BridgeSubscriptionPreviewWorkKind::Continuation),
        crate::facade::BridgeSubscriptionPreviewResidueCategory::SignalVisible => work_trace
            .record_digest_for(crate::facade::BridgeSubscriptionPreviewWorkKind::Diagnostics),
    }
}

#[test]
fn preview_discard_rejects_nonzero_authoritative_residue() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active) =
        preview_active_detail_subscription("discard-nonzero", resource_request);
    let residue_index = runtime.build_subscription_preview_residue_scope_index(
        &preview_active,
        preview_residue_inputs_with_count(
            &runtime,
            &preview_active,
            crate::facade::BridgeSubscriptionPreviewResidueCategory::AuthoritativeTruthSubscription,
            1,
        ),
    );

    let rejection = runtime
        .prove_preview_scope_discard_residue(preview_active, residue_index)
        .expect_err("nonzero authoritative residue must reject discard");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewDiscardResidueRejectionKind::NonzeroResidue
    );
    let nonzero_categories = rejection.rejection_context().nonzero_categories();
    assert_eq!(nonzero_categories.len(), 1);
    assert_eq!(
        nonzero_categories[0].category(),
        crate::facade::BridgeSubscriptionPreviewResidueCategory::AuthoritativeTruthSubscription
    );
    assert_eq!(nonzero_categories[0].residue_count(), 1);
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_residue_check_count(),
        7
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_residue_nonzero_count(),
        1
    );
}

#[test]
fn preview_discard_rejects_missing_residue_category() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active) =
        preview_active_detail_subscription("discard-missing-category", resource_request);
    let residue_inputs = zero_preview_residue_inputs(&runtime, &preview_active)
        .into_iter()
        .filter(|input| {
            input.category()
                != crate::facade::BridgeSubscriptionPreviewResidueCategory::SignalVisible
        })
        .collect();
    let residue_index =
        runtime.build_subscription_preview_residue_scope_index(&preview_active, residue_inputs);

    let rejection = runtime
        .prove_preview_scope_discard_residue(preview_active, residue_index)
        .expect_err("missing residue categories must reject discard");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewDiscardResidueRejectionKind::MissingResidueCategory
    );
    assert_eq!(
        rejection.rejection_context().missing_category_value(),
        Some(crate::facade::BridgeSubscriptionPreviewResidueCategory::SignalVisible)
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_residue_check_count(),
        6
    );
}

#[test]
fn preview_discard_rejects_duplicate_residue_category() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active) =
        preview_active_detail_subscription("discard-duplicate-category", resource_request);
    let mut residue_inputs = zero_preview_residue_inputs(&runtime, &preview_active);
    residue_inputs.push(
        crate::facade::BridgeSubscriptionPreviewResidueArtifactInput::zero_from_preview_work_trace(
            crate::facade::BridgeSubscriptionPreviewResidueCategory::SignalVisible,
            &preview_work_trace(&runtime, &preview_active, "discard-duplicate-category"),
        ),
    );
    let residue_index =
        runtime.build_subscription_preview_residue_scope_index(&preview_active, residue_inputs);

    let rejection = runtime
        .prove_preview_scope_discard_residue(preview_active, residue_index)
        .expect_err("duplicate residue categories must reject discard");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewDiscardResidueRejectionKind::DuplicateResidueCategory
    );
    assert_eq!(
        rejection.rejection_context().duplicate_category_value(),
        Some(crate::facade::BridgeSubscriptionPreviewResidueCategory::SignalVisible)
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_residue_check_count(),
        8
    );
}

#[test]
fn preview_discard_rejects_scope_index_drift_before_residue_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, preview_active) =
        preview_active_detail_subscription("discard-scope-a", resource_request);
    let (other_runtime, other_preview_active) =
        preview_active_detail_subscription("discard-scope-b", resource_request);
    let other_residue_index = other_runtime.build_subscription_preview_residue_scope_index(
        &other_preview_active,
        zero_preview_residue_inputs(&other_runtime, &other_preview_active),
    );

    let rejection = runtime
        .prove_preview_scope_discard_residue(preview_active, other_residue_index)
        .expect_err("residue index from another preview active must reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionPreviewDiscardResidueRejectionKind::PreviewActiveMismatch
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_preview_residue_check_count(),
        0
    );
}

#[test]
fn detail_and_collection_preview_subscriptions_share_residue_proof_path() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (detail_runtime, detail_preview_active) =
        preview_active_detail_subscription("discard-detail", resource_request);
    let detail_active_identity = detail_preview_active
        .preview_active_subscription_identity()
        .clone();
    let detail_index = detail_runtime.build_subscription_preview_residue_scope_index(
        &detail_preview_active,
        zero_preview_residue_inputs(&detail_runtime, &detail_preview_active),
    );
    let detail_proof = detail_runtime
        .prove_preview_scope_discard_residue(detail_preview_active, detail_index)
        .expect("detail preview discard should prove zero residue");

    let (collection_runtime, collection_preview_active) =
        preview_active_collection_subscription("discard-collection", resource_request);
    let collection_active_identity = collection_preview_active
        .preview_active_subscription_identity()
        .clone();
    let collection_index = collection_runtime.build_subscription_preview_residue_scope_index(
        &collection_preview_active,
        zero_preview_residue_inputs(&collection_runtime, &collection_preview_active),
    );
    let collection_proof = collection_runtime
        .prove_preview_scope_discard_residue(collection_preview_active, collection_index)
        .expect("collection preview discard should prove zero residue");

    assert_eq!(detail_proof.total_residue_count(), 0);
    assert_eq!(collection_proof.total_residue_count(), 0);
    assert_eq!(detail_proof.artifact_records().len(), 7);
    assert_eq!(collection_proof.artifact_records().len(), 7);
    assert_ne!(
        detail_proof.preview_active_subscription_identity(),
        collection_proof.preview_active_subscription_identity()
    );
    assert_eq!(
        detail_proof.preview_active_subscription_identity(),
        &detail_active_identity
    );
    assert_eq!(
        collection_proof.preview_active_subscription_identity(),
        &collection_active_identity
    );
}
