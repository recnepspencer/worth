use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::super::super::support::*;

#[derive(Clone)]
struct PhaseNineMaintainer {
    invocations: Arc<AtomicUsize>,
}

impl WorthQueryDerivedViewMaintainer for PhaseNineMaintainer {
    fn maintain(
        &mut self,
        view: &crate::program::WorthQueryDerivedView,
        _delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let next = self.invocations.fetch_add(1, Ordering::SeqCst) + 1;
        let retained_row = retained_string_test_row("title.value", "Phase Nine");
        materialization.replace_retained_rows([retained_row.clone()]);
        WorthQueryDerivedPatch::whole_refresh_materialized(
            view.name(),
            crate::memory_workspace::admit_external_commit_label(format!(
                "phase-nine-refresh-{next}"
            )),
            [test_aspect_touch("title.value")],
            WorthQueryDerivedPatchPayload::from_retained_row(retained_row),
            format!("phase-nine-publication-{next}"),
        )
    }
}

#[test]
fn phase_nine_facade_families_are_supported_and_admitted() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.phase-nine-support")
        .expect("workspace should build");

    let contract = workspace.public_api_contract();
    for family in [
        WorthQueryRuntimeFacadeFamily::Read,
        WorthQueryRuntimeFacadeFamily::Live,
        WorthQueryRuntimeFacadeFamily::Computed,
        WorthQueryRuntimeFacadeFamily::Effect,
        WorthQueryRuntimeFacadeFamily::BranchPreview,
        WorthQueryRuntimeFacadeFamily::Write,
        WorthQueryRuntimeFacadeFamily::Inspect,
    ] {
        assert_eq!(
            contract
                .family(family)
                .expect("legacy family must be present")
                .owner_closure(),
            "Milestone 9.3"
        );
    }

    for family in [
        WorthQueryRuntimeFacadeFamily::SharedRead,
        WorthQueryRuntimeFacadeFamily::Submission,
    ] {
        let support = contract
            .family(family)
            .expect("phase-nine family must be present");
        assert_eq!(
            support.status(),
            WorthQueryRuntimeFamilySupportStatus::Supported
        );
        assert_eq!(
            support.teaching_posture(),
            WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx
        );
        assert_eq!(support.owner_closure(), "Milestone 9.7");
        assert!(!support.admission_fail_closed());
        workspace
            .admit_public_api_family(family)
            .expect("supported phase-nine family should admit");
    }
}

#[test]
fn workspace_submission_lane_matches_existing_workspace_convenience_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let command = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("identity.id"),
            test_authored_string_aspect_value("task-1"),
        )
        .set_aspect(
            test_aspect_touch("title.value"),
            test_authored_string_aspect_value("Phase Nine"),
        )
        .build_insert("Task")
        .expect("insert command should build");

    let convenience_receipt = stateful_bridge_task_runtime(request_execution)
        .workspace("task.phase-nine-convenience")
        .expect("workspace should build")
        .write(request_execution, command.clone())
        .expect("workspace convenience write should succeed");
    let submission_receipt = {
        let mut workspace = stateful_bridge_task_runtime(request_execution)
            .workspace("task.phase-nine-submission")
            .expect("workspace should build");
        workspace
            .submissions()
            .expect("submission lane should mint")
            .submit(request_execution, command)
            .expect("submission lane write should succeed")
    };

    assert_eq!(
        convenience_receipt.commit_identity(),
        submission_receipt.commit_identity()
    );
    assert_eq!(
        mutation_receipt_summary(&convenience_receipt),
        mutation_receipt_summary(&submission_receipt)
    );
}

#[test]
fn workspace_shared_read_lane_matches_runtime_owned_context_without_recomputation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.phase-nine-shared-read")
        .expect("workspace should build");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view_request(
            request_execution,
            "tasks.phase-nine",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let invocations = Arc::new(AtomicUsize::new(0));
    let derived: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed_view(
            request_execution,
            crate::program::WorthQueryDerivedView::new(
                "derived.phase-nine",
                [test_aspect_touch("title.value")],
            )
            .depends_on_live(&live),
            PhaseNineMaintainer {
                invocations: Arc::clone(&invocations),
            },
        )
        .expect("derived view should declare");

    workspace
        .insert(request_execution, "Task", |builder| {
            builder
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-1"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Phase Nine"),
                )
        })
        .expect("insert should publish derived artifact");

    let facade_artifact = workspace
        .shared_read_context(request_execution)
        .expect("workspace shared read context should mint")
        .published_derived_artifact(&derived)
        .expect("workspace shared read artifact should mint");
    let decomposed_artifact =
        WorthQuerySharedReadContext::from_runtime(request_execution, &workspace.runtime)
            .published_derived_artifact(&derived)
            .expect("runtime shared read artifact should mint");

    assert_eq!(facade_artifact, decomposed_artifact);
    assert_eq!(
        invocations.load(Ordering::SeqCst),
        1,
        "shared read parity must not trigger recomputation"
    );
}

#[derive(Debug, PartialEq, Eq)]
struct MutationReceiptSummary {
    snapshot_identity: String,
    authority_lane: String,
    mutation_family: String,
    declared_collection: Option<String>,
    declared_entity_identity: Option<String>,
    target_collection: Option<String>,
    target_entity_identity: Option<String>,
    declared_aspect_value_digest: Option<String>,
    affected_live_view_ids: Vec<String>,
    affected_derived_view_ids: Vec<String>,
    pending_write_intent_count: usize,
    suppressed_effect_count: usize,
    meaningful_effect_suppression_count: usize,
    effect_expression_failure_count: usize,
    refresh_fallback: bool,
    deltas: Vec<MutationDeltaSummary>,
}

#[derive(Debug, PartialEq, Eq)]
struct MutationDeltaSummary {
    collection: String,
    entity_identity: String,
    kind: crate::memory_workspace::WorthQueryMutationKind,
    touched_aspects: Vec<WorthQueryAspectTouch>,
}

fn mutation_receipt_summary(receipt: &WorthQueryWriteReceipt) -> MutationReceiptSummary {
    MutationReceiptSummary {
        snapshot_identity: receipt.snapshot_evidence_identity().as_str().to_string(),
        authority_lane: receipt.authority_lane().as_str().to_string(),
        mutation_family: receipt.mutation_family().as_str().to_string(),
        declared_collection: receipt
            .terminal_declared_collection_projection()
            .map(str::to_string),
        declared_entity_identity: receipt
            .declared_entity_identity()
            .map(|identity| identity.terminal_projection_for_reporting().to_string()),
        target_collection: receipt
            .terminal_target_collection_projection()
            .map(str::to_string),
        target_entity_identity: receipt
            .target_entity_identity()
            .map(|identity| identity.terminal_projection_for_reporting().to_string()),
        declared_aspect_value_digest: receipt.declared_aspect_value_digest().map(str::to_string),
        affected_live_view_ids: receipt
            .terminal_affected_live_view_ids_projection()
            .to_vec(),
        affected_derived_view_ids: receipt
            .terminal_affected_derived_view_ids_projection()
            .to_vec(),
        pending_write_intent_count: receipt.pending_write_intent_count(),
        suppressed_effect_count: receipt.suppressed_effect_count(),
        meaningful_effect_suppression_count: receipt.meaningful_effect_suppression_count(),
        effect_expression_failure_count: receipt.effect_expression_failure_count(),
        refresh_fallback: receipt.refresh_fallback(),
        deltas: receipt
            .deltas()
            .iter()
            .map(|delta| MutationDeltaSummary {
                collection: delta.collection().to_string(),
                entity_identity: delta
                    .entity_identity()
                    .terminal_projection_for_reporting()
                    .to_string(),
                kind: delta.kind().clone(),
                touched_aspects: delta.admitted_touched_aspects().to_vec(),
            })
            .collect(),
    }
}
