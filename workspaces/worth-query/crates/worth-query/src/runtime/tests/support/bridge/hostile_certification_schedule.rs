use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::projection_consumption::ProjectionAuthorityOutcome;
use crate::runtime::tests::support::*;
use worth_foundational::facade::InternedString;

mod schedule;

use schedule::*;

#[derive(Clone)]
struct HostileCertificationMaintainer {
    invocations: Arc<AtomicUsize>,
    titles: &'static [&'static str],
}

impl WorthQueryDerivedViewMaintainer for HostileCertificationMaintainer {
    fn maintain(
        &mut self,
        view: &crate::program::WorthQueryDerivedView,
        _delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let next = self.invocations.fetch_add(1, Ordering::SeqCst);
        let title = self
            .titles
            .get(next)
            .copied()
            .unwrap_or(self.titles[self.titles.len() - 1]);
        let retained_row = retained_string_test_row("title.value", title);
        materialization.replace_retained_rows([retained_row.clone()]);
        WorthQueryDerivedPatch::whole_refresh_materialized(
            view.name(),
            crate::memory_workspace::admit_external_commit_label(format!(
                "hostile-certification-refresh-{}",
                next + 1
            )),
            [test_aspect_touch("title.value")],
            WorthQueryDerivedPatchPayload::from_retained_row(retained_row),
            format!("hostile-certification-publication-{}", next + 1),
        )
    }
}

struct HostileExecutionState {
    workspace: WorthQueryWorkspace,
    derived: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape>,
    invocations: Arc<AtomicUsize>,
    receipt_summaries: Vec<String>,
    reader_results: Vec<String>,
    published_artifacts: Vec<String>,
    preview_closeouts: Vec<String>,
    branch_bases: Vec<String>,
    authoritative_receipts: Vec<WorthQueryWriteReceipt>,
    reader_derived_evaluation_count: usize,
    delivery_residue_count: usize,
    first_artifact: Option<WorthQueryPublishedDerivedArtifactHandle>,
    second_artifact: Option<WorthQueryPublishedDerivedArtifactHandle>,
    third_artifact: Option<WorthQueryPublishedDerivedArtifactHandle>,
}

pub(in crate::runtime::tests) fn execute_runtime_hostile_schedule(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> RuntimeHostileCertificationArtifact {
    let schedule = hostile_schedule();
    run_hostile_schedule_steps(request_execution, schedule.steps.iter().copied())
}

pub(in crate::runtime::tests) fn replay_runtime_hostile_schedule(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> RuntimeHostileCertificationArtifact {
    let schedule = hostile_schedule();
    let recorded_steps = schedule.steps.to_vec();
    run_hostile_schedule_steps(request_execution, recorded_steps)
}

fn run_hostile_schedule_steps(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    steps: impl IntoIterator<Item = HostileScheduleStep>,
) -> RuntimeHostileCertificationArtifact {
    let mut state = HostileExecutionState::new(request_execution);
    for step in steps {
        state.apply(request_execution, step);
    }
    state.finish()
}

impl HostileExecutionState {
    fn new(request_execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        let mut workspace = stateful_bridge_task_runtime(request_execution)
            .workspace("runtime.tests.hostile-certification")
            .expect("workspace should build");
        let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
            .live_view_request(
                request_execution,
                "tasks.hostile-certification",
                task_live_request(),
                task_schema(),
            )
            .expect("live view should declare");
        let invocations = Arc::new(AtomicUsize::new(0));
        let derived = workspace
            .computed_view(
                request_execution,
                crate::program::WorthQueryDerivedView::new(
                    "derived.hostile-certification",
                    [test_aspect_touch("title.value")],
                )
                .depends_on_live(&live),
                HostileCertificationMaintainer {
                    invocations: Arc::clone(&invocations),
                    titles: &["Task One", "Task Two", "Task Three"],
                },
            )
            .expect("derived view should declare");
        Self {
            workspace,
            derived,
            invocations,
            receipt_summaries: Vec::new(),
            reader_results: Vec::new(),
            published_artifacts: Vec::new(),
            preview_closeouts: Vec::new(),
            branch_bases: Vec::new(),
            authoritative_receipts: Vec::new(),
            reader_derived_evaluation_count: 0,
            delivery_residue_count: 0,
            first_artifact: None,
            second_artifact: None,
            third_artifact: None,
        }
    }

    fn apply(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: HostileScheduleStep,
    ) {
        match step {
            HostileScheduleStep::ConsumeUnpublishedDerived => {
                self.capture_unpublished_posture(request_execution)
            }
            HostileScheduleStep::OpenBranch(label) => {
                self.record_branch_basis(request_execution, label)
            }
            HostileScheduleStep::DiscardPreview {
                label,
                identity,
                title,
            } => self.record_preview_discard(request_execution, label, identity, title),
            HostileScheduleStep::SubmitTask {
                identity,
                title,
                slot,
            } => self.record_submission_publication(request_execution, identity, title, slot),
            HostileScheduleStep::ReconsumePublishedArtifacts {
                current_slot,
                stable_slot,
            } => self.reconsume_published_artifacts(current_slot, stable_slot),
            HostileScheduleStep::PromotePreview {
                label,
                identity,
                title,
            } => self.record_preview_promotion(request_execution, label, identity, title),
        }
    }

    fn finish(mut self) -> RuntimeHostileCertificationArtifact {
        self.first_artifact = None;
        self.second_artifact = None;
        self.third_artifact = None;
        let counters = RuntimeHostileCertificationCounters::for_runtime(
            &self.workspace.runtime,
            hostile_journal_gap_count(&self.authoritative_receipts),
            self.reader_derived_evaluation_count,
            self.delivery_residue_count,
        );
        RuntimeHostileCertificationArtifact::new(
            self.receipt_summaries,
            self.reader_results,
            self.published_artifacts,
            self.preview_closeouts,
            self.branch_bases,
            counters,
        )
    }

    fn capture_unpublished_posture(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        let unpublished = self
            .workspace
            .shared_read_context(request_execution)
            .expect("shared read context should mint")
            .published_derived_artifact(&self.derived)
            .expect("declared derived handle should mint");
        let pending_state = match hostile_consume_title_attempt(&unpublished) {
            WorthQueryPublishedProjectionAuthorityOutcome::ResultState(state) => state,
            other => panic!("expected pending published artifact posture, got {other:?}"),
        };
        self.reader_results
            .push(pending_state.result_state_for_reporting().to_string());
        self.published_artifacts
            .push(hostile_published_artifact_digest(&unpublished, None));
    }

    fn record_branch_basis(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: &'static str,
    ) {
        let branch = self
            .workspace
            .branch(request_execution, test_session_label(label))
            .expect("branch churn should admit");
        self.branch_bases.push(hostile_branch_basis_digest(&branch));
    }

    fn record_preview_discard(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: &'static str,
        identity: &'static str,
        title: &'static str,
    ) {
        let discarded = {
            let mut preview = self
                .workspace
                .preview(request_execution, test_session_label(label))
                .expect("preview churn should admit");
            preview
                .insert(request_execution, "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value(identity),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value(title),
                    )
                })
                .expect("preview staging should succeed");
            preview.discard()
        };
        self.delivery_residue_count += hostile_preview_delivery_residue_count(&discarded);
        self.preview_closeouts
            .push(hostile_preview_closeout_digest(&discarded));
    }

    fn record_submission_publication(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &'static str,
        title: &'static str,
        slot: PublishedArtifactSlot,
    ) {
        let receipt = self
            .workspace
            .submissions()
            .expect("submission lane should mint")
            .submit(
                request_execution,
                hostile_insert_task_command(identity, title),
            )
            .expect("submission should succeed");
        self.receipt_summaries
            .push(hostile_write_receipt_digest(&receipt));
        self.authoritative_receipts.push(receipt);

        let artifact = self
            .workspace
            .shared_read_context(request_execution)
            .expect("shared read context should mint")
            .published_derived_artifact(&self.derived)
            .expect("published artifact should mint");
        let consumed_title = self.consume_title_without_recompute(&artifact);
        self.reader_results.push(consumed_title.clone());
        self.published_artifacts
            .push(hostile_published_artifact_digest(
                &artifact,
                Some(consumed_title.as_str()),
            ));
        self.replace_artifact_slot(slot, artifact);
    }

    fn reconsume_published_artifacts(
        &mut self,
        current_slot: PublishedArtifactSlot,
        stable_slot: Option<PublishedArtifactSlot>,
    ) {
        if let Some(stable_slot) = stable_slot {
            let stable_artifact = self.artifact_slot(stable_slot).clone();
            let stable_title = self.consume_title_without_recompute(&stable_artifact);
            self.reader_results.push(stable_title);
        }
        let current_artifact = self.artifact_slot(current_slot).clone();
        let current_title = self.consume_title_without_recompute(&current_artifact);
        self.reader_results.push(current_title.clone());
        self.published_artifacts
            .push(hostile_published_artifact_digest(
                &current_artifact,
                Some(current_title.as_str()),
            ));
    }

    fn record_preview_promotion(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: &'static str,
        identity: &'static str,
        title: &'static str,
    ) {
        let promoted = {
            let mut preview = self
                .workspace
                .preview(request_execution, test_session_label(label))
                .expect("preview churn should admit");
            preview
                .insert(request_execution, "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value(identity),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value(title),
                    )
                })
                .expect("preview promotion staging should succeed");
            preview
                .promote(request_execution)
                .expect("preview promotion should succeed")
        };
        self.delivery_residue_count += hostile_preview_delivery_residue_count(&promoted);
        self.preview_closeouts
            .push(hostile_preview_closeout_digest(&promoted));
        let promoted_artifact = self
            .workspace
            .shared_read_context(request_execution)
            .expect("shared read context should mint")
            .published_derived_artifact(&self.derived)
            .expect("third published artifact should mint");
        self.replace_artifact_slot(PublishedArtifactSlot::Third, promoted_artifact);
    }

    fn artifact_slot(
        &self,
        slot: PublishedArtifactSlot,
    ) -> &WorthQueryPublishedDerivedArtifactHandle {
        match slot {
            PublishedArtifactSlot::First => self.first_artifact.as_ref(),
            PublishedArtifactSlot::Second => self.second_artifact.as_ref(),
            PublishedArtifactSlot::Third => self.third_artifact.as_ref(),
        }
        .expect("requested published artifact slot should exist")
    }

    fn replace_artifact_slot(
        &mut self,
        slot: PublishedArtifactSlot,
        artifact: WorthQueryPublishedDerivedArtifactHandle,
    ) {
        match slot {
            PublishedArtifactSlot::First => self.first_artifact = Some(artifact),
            PublishedArtifactSlot::Second => self.second_artifact = Some(artifact),
            PublishedArtifactSlot::Third => self.third_artifact = Some(artifact),
        }
    }

    fn consume_title_without_recompute(
        &mut self,
        artifact: &WorthQueryPublishedDerivedArtifactHandle,
    ) -> String {
        let before = self.invocations.load(Ordering::SeqCst);
        let title = match hostile_consume_title_attempt(artifact) {
            WorthQueryPublishedProjectionAuthorityOutcome::Current(
                ProjectionAuthorityOutcome::Admitted(completed),
            ) => completed
                .facts()
                .display_fields()
                .first()
                .and_then(|fact| match fact.native_value().scalar()? {
                    AspectValue::String(InternedString::Raw(value)) => Some(value.as_str()),
                    AspectValue::String(InternedString::Symbol(_)) => None,
                    _ => None,
                })
                .expect("display title should be present")
                .to_string(),
            other => panic!("expected admitted published fact consumption, got {other:?}"),
        };
        let after = self.invocations.load(Ordering::SeqCst);
        self.reader_derived_evaluation_count += after.saturating_sub(before);
        title
    }
}
