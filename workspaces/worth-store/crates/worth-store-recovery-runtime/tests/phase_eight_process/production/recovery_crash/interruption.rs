use std::path::{Path, PathBuf};
use std::time::Duration;

use worth_store::physical_runtime::PhysicalRecoveryYieldpointStage;
use worth_store_recovery_runtime::{
    RecoveryReportDenialCause, RecoveryReportEnvelope, RecoveryReportOutcome,
};

use super::super::super::comparison;
use super::super::harness::ProcessWorld;

#[path = "interruption/cancelled.rs"]
mod cancelled;
#[path = "interruption/deadline.rs"]
mod deadline;
#[path = "interruption/publication_adjudication.rs"]
mod publication_adjudication;

const RECOVERY_INTERRUPTION_MEMORY_BUDGET_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn run_publication(world: &ProcessWorld, index: usize) {
    run_stages(world, index, |stage| !super::is_cleanup_stage(stage));
    run_deadlines(world, index + super::RECOVERY_SEAMS.len(), |stage| {
        !super::is_cleanup_stage(stage)
    });
}

pub(super) fn run_cleanup(world: &ProcessWorld, index: usize) {
    run_stages(world, index, super::is_cleanup_stage);
    run_deadlines(
        world,
        index + super::RECOVERY_SEAMS.len(),
        super::is_cleanup_stage,
    );
}

fn run_stages(
    world: &ProcessWorld,
    index: usize,
    include: impl Fn(PhysicalRecoveryYieldpointStage) -> bool,
) {
    for (offset, stage) in super::RECOVERY_SEAMS
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, stage)| include(*stage))
    {
        run_cancelled(world, index + offset, stage);
    }
}

fn run_cancelled(world: &ProcessWorld, index: usize, stage: PhysicalRecoveryYieldpointStage) {
    let root = copied_root(world, index, stage, "cancelled");
    let publication_before =
        (!super::is_cleanup_stage(stage)).then(|| world.parent_history_root(&root));
    let cleanup_before = super::is_cleanup_stage(stage).then(|| {
        world
            .cleanup_candidate(&root)
            .unwrap_or_else(|error| panic!("cleanup cancellation candidate precondition: {error}"))
    });
    let parent = root.parent().expect("cancelled recovery parent");
    let name = format!("cancelled-{index}-{}", stage.label());
    let report = parent.join(format!("{name}-report.bin"));
    let reached = parent.join(format!("{name}-reached"));
    let release = parent.join(format!("{name}-release"));
    let cancel = release.with_extension("cancel");
    let mut child = cancelled::spawn(
        &root,
        &report,
        world.parent_path(),
        stage,
        &reached,
        &release,
    );
    super::seam::wait_for_reached(&mut child, &reached, stage);
    std::fs::write(&cancel, "cancel").expect("arm product cancellation");
    let output = child
        .wait_with_output_within(Duration::from_secs(120))
        .expect("wait for product cancellation");
    let report = decode_report(&report, "cancelled recovery");
    if super::is_cleanup_stage(stage) {
        assert!(
            output.status.success(),
            "cleanup cancellation must preserve a successful recovered handoff at {stage:?}: status={:?}, outcome={:?}, cause={:?}",
            output.status,
            report.outcome(),
            report.denial_cause(),
        );
        assert_eq!(report.outcome(), RecoveryReportOutcome::Recovered);
        assert!(
            report.counters().cleanup_deferred() > 0,
            "cleanup cancellation must defer at least one cleanup action"
        );
        assert_cleanup_outcome(
            world,
            &root,
            cleanup_before.as_ref().expect("cleanup candidate"),
            report.counters().cleanup_performed(),
            "cancelled",
        );
        let observer = world.observe_root(&root, &format!("cancelled-{index}-observer"));
        let history = world.parent_history_after_recovery(&root);
        comparison::compare_runtime_and_observer_with_budget(
            &report,
            &observer.report,
            &history,
            RECOVERY_INTERRUPTION_MEMORY_BUDGET_BYTES,
        )
        .unwrap_or_else(|error| {
            panic!("cancelled cleanup evidence diverged at {stage:?}: {error:?}")
        });
        return;
    }
    assert!(
        !output.status.success(),
        "product cancellation must not succeed"
    );
    assert!(!matches!(
        report.outcome(),
        RecoveryReportOutcome::Recovered
    ));
    assert!(
        matches!(
            report.denial_cause(),
            Some(RecoveryReportDenialCause::Blocked(_))
                | Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate)
        ),
        "product cancellation at {stage:?} must preserve a typed interruption: {:?}",
        report.denial_cause()
    );
    let publication_after = world.parent_history_after_recovery(&root);
    let publication_observer = world.observe_root(&root, &format!("cancelled-{index}-observer"));
    publication_adjudication::adjudicate(
        &root,
        &report,
        publication_before
            .as_ref()
            .expect("publication cancellation baseline history"),
        &publication_after,
        &publication_observer.report,
        "cancelled",
    );
}

fn run_deadlines(
    world: &ProcessWorld,
    index: usize,
    include: impl Fn(PhysicalRecoveryYieldpointStage) -> bool,
) {
    for (offset, stage) in super::RECOVERY_SEAMS
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, stage)| include(*stage))
    {
        run_deadline(world, index + offset, stage);
    }
}

fn run_deadline(world: &ProcessWorld, index: usize, stage: PhysicalRecoveryYieldpointStage) {
    let root = copied_root(world, index, stage, "deadline");
    let publication_before =
        (!super::is_cleanup_stage(stage)).then(|| world.parent_history_root(&root));
    let cleanup_before = super::is_cleanup_stage(stage).then(|| {
        world
            .cleanup_candidate(&root)
            .unwrap_or_else(|error| panic!("cleanup deadline candidate precondition: {error}"))
    });
    let parent = root.parent().expect("deadline recovery parent");
    let name = format!("deadline-{index}-{}", stage.label());
    let report = parent.join(format!("{name}-report.bin"));
    let reached = parent.join(format!("{name}-reached"));
    let release = parent.join(format!("{name}-release"));
    let cancel = release.with_extension("cancel");
    let mut child = deadline::spawn(
        &root,
        &report,
        world.parent_path(),
        stage,
        &reached,
        &release,
    );
    super::seam::wait_for_reached(&mut child, &reached, stage);
    let output = child
        .wait_with_output_within(Duration::from_secs(120))
        .expect("wait for product deadline");
    let report = decode_report(&report, "deadline recovery");
    assert!(
        !cancel.exists(),
        "deadline proof must not arm the cancellation file"
    );
    if super::is_cleanup_stage(stage) {
        assert!(
            output.status.success(),
            "cleanup deadline must preserve a successful recovered handoff at {stage:?}: status={:?}, outcome={:?}, cause={:?}",
            output.status,
            report.outcome(),
            report.denial_cause(),
        );
        assert_eq!(report.outcome(), RecoveryReportOutcome::Recovered);
        assert!(report.counters().cleanup_deferred() > 0);
        assert_cleanup_outcome(
            world,
            &root,
            cleanup_before.as_ref().expect("cleanup candidate"),
            report.counters().cleanup_performed(),
            "deadline",
        );
        let observer = world.observe_root(&root, &format!("deadline-{index}-observer"));
        let history = world.parent_history_after_recovery(&root);
        comparison::compare_runtime_and_observer_with_budget(
            &report,
            &observer.report,
            &history,
            RECOVERY_INTERRUPTION_MEMORY_BUDGET_BYTES,
        )
        .unwrap_or_else(|error| {
            panic!("deadline cleanup evidence diverged at {stage:?}: {error:?}")
        });
        return;
    }
    assert!(
        !output.status.success(),
        "product deadline must not succeed at {stage:?}"
    );
    assert!(
        matches!(
            report.denial_cause(),
            Some(RecoveryReportDenialCause::Blocked(_))
                | Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate)
        ),
        "product deadline at {stage:?} must preserve a typed interruption: {:?}",
        report.denial_cause()
    );
    let publication_after = world.parent_history_after_recovery(&root);
    let publication_observer = world.observe_root(&root, &format!("deadline-{index}-observer"));
    publication_adjudication::adjudicate(
        &root,
        &report,
        publication_before
            .as_ref()
            .expect("publication deadline baseline history"),
        &publication_after,
        &publication_observer.report,
        "deadline",
    );
}

fn decode_report(path: &Path, label: &str) -> RecoveryReportEnvelope {
    RecoveryReportEnvelope::decode(
        &std::fs::read(path).unwrap_or_else(|error| panic!("{label} report bytes: {error}")),
    )
    .unwrap_or_else(|error| panic!("{label} report decode: {error:?}"))
}

fn assert_cleanup_outcome(
    world: &ProcessWorld,
    root: &Path,
    before: &super::super::super::history::CleanupCandidateProof,
    cleanup_performed: u64,
    posture: &str,
) {
    if world.verify_cleanup_preserved(root, before).is_ok() {
        assert_eq!(
            cleanup_performed, 0,
            "{posture} cleanup retained the candidate but reported a performed removal"
        );
        return;
    }
    let transition = world
        .verify_cleanup_transition(root, before)
        .unwrap_or_else(|error| {
            panic!("{posture} cleanup changed the candidate dishonestly: {error}")
        });
    assert_eq!(
        transition.removed_covered.len() as u64,
        before.covered_segments
    );
    assert_eq!(transition.retained, before.retained);
}

fn copied_root(
    world: &ProcessWorld,
    index: usize,
    stage: PhysicalRecoveryYieldpointStage,
    label: &str,
) -> PathBuf {
    let root = world
        .writer
        .root
        .parent()
        .expect("recovery source parent")
        .join(format!("recovery-{label}-{index}-{}", stage.label()));
    copy_directory(&world.writer.root, &root);
    root
}

fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("create product interruption root");
    for entry in std::fs::read_dir(source).expect("read product interruption source") {
        let entry = entry.expect("read product interruption entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path)
                .expect("copy product interruption artifact");
        }
    }
}
