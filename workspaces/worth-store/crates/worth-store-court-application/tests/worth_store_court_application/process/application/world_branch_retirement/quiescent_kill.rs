//! Quiescent process-kill journeys for durable branch creation, retirement, and disposal.
use super::{create_branch, open_home, publish_round};
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;

const LOCATION: &str = "WORTH_C28_KILL_LOCATION";
const MARKERS: &str = "WORTH_C28_KILL_MARKERS";
const SCENARIO: &str = "WORTH_C28_KILL_SCENARIO";
const CHILD: &str =
    "process::application::world_branch_retirement::quiescent_kill::c28_quiescent_child";

pub(super) fn run_killed(scenario: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let location = root.path().join("home");
    let markers = root.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[
                (LOCATION, location.as_path()),
                (MARKERS, markers.as_path()),
                (SCENARIO, std::path::Path::new(scenario)),
            ],
            &markers.join("ready"),
            &markers.join("finished"),
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    root
}

#[test]
fn durable_branch_creation_survives_quiescent_kill_and_reopen() {
    let root = run_killed("creation");
    let expected: Vec<String> = std::fs::read_to_string(root.path().join("markers/creation"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(expected.len(), 3);
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let reopened = open_home(&root.path().join("home"), &execution);
    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    assert_eq!(installed.branches().len(), 4);
    for row in &expected {
        let parts: Vec<&str> = row.split('|').collect();
        let branch = installed
            .branches()
            .iter()
            .find(|b| b.branch_identity().name().as_str() == parts[0])
            .expect("forked branch must survive quiescent kill");
        assert_eq!(branch.selected_commit().ordinal().to_string(), parts[1]);
        assert_eq!(branch.reference_generation().get().to_string(), parts[2]);
        assert_eq!(
            branch
                .basis()
                .relational_basis()
                .descriptor()
                .branch_id()
                .0,
            parts[3]
        );
        assert_eq!(
            format!("{:?}", branch.basis().signal_basis().branch_id()),
            parts[4]
        );
    }
    let both = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "both-fork")
        .unwrap();
    let trace = reopened
        .home
        .world()
        .observation_port()
        .trace_product_branch_ancestry(&execution, both.lifecycle_incarnation(), None)
        .unwrap();
    assert_eq!(trace.commits().len(), 2);
    drop((trace, installed));
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}

#[test]
fn durable_branch_retirement_survives_quiescent_kill_and_keeps_history_readable() {
    let root = run_killed("retirement");
    let retired_ordinal: u64 = std::fs::read_to_string(root.path().join("markers/ordinal"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let reopened = open_home(&root.path().join("home"), &execution);
    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    assert_eq!(installed.branches().len(), 1);
    let durable_owner = reopened.home.world().owner_identity().durable().unwrap();
    let retired_commit =
        worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            durable_owner,
            retired_ordinal,
        );
    let readmitted = reopened
        .home
        .world()
        .observation_port()
        .readmitted_publication(&retired_commit)
        .unwrap();
    assert_eq!(readmitted.after().selected_commit().ordinal(), retired_ordinal);
    drop((readmitted, installed));
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}

#[test]
fn durable_history_disposal_survives_quiescent_kill_without_resurrection() {
    let root = run_killed("disposal");
    let disposed_ordinal: u64 = std::fs::read_to_string(root.path().join("markers/ordinal"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let reopened = open_home(&root.path().join("home"), &execution);
    assert_eq!(
        reopened
            .home
            .world()
            .inspection_port()
            .history_snapshot()
            .unwrap()
            .installed_commits(),
        1
    );
    let durable_owner = reopened.home.world().owner_identity().durable().unwrap();
    let disposed_commit =
        worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            durable_owner,
            disposed_ordinal,
        );
    assert!(matches!(
        reopened
            .home
            .world()
            .observation_port()
            .readmitted_publication(&disposed_commit),
        Err(RuntimeWorldServiceDenial::Denied(
            WorldReadmittedPublicationDenial::NotReadmitted
        ))
    ));
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}

#[test]
fn retiring_parent_with_live_child_and_name_reuse_survives_quiescent_kill() {
    let root = run_killed("parent-retired");
    let fields: Vec<String> = std::fs::read_to_string(root.path().join("markers/parent-retired"))
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    assert_eq!(fields.len(), 3);
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let reopened = open_home(&root.path().join("home"), &execution);
    let installed = reopened
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(&execution)
        .unwrap();
    assert_eq!(installed.branches().len(), 3);
    let child = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "child-b")
        .expect("live child must survive retired parent across quiescent kill");
    assert_eq!(child.selected_commit().ordinal().to_string(), fields[0]);
    let reused = installed
        .branches()
        .iter()
        .find(|b| b.branch_identity().name().as_str() == "parent-a")
        .expect("later same-name incarnation must survive quiescent kill");
    assert_eq!(
        reused.lifecycle_incarnation().ordinal().to_string(),
        fields[2]
    );
    assert_ne!(
        reused.lifecycle_incarnation().ordinal().to_string(),
        fields[1]
    );
    drop(installed);
    crate::process::application::world_occurrence_readmission::close(reopened.home, &execution);
}

#[test]
fn c28_quiescent_child() {
    let Some(scenario) = std::env::var_os(SCENARIO) else {
        return;
    };
    let location = std::path::PathBuf::from(std::env::var_os(LOCATION).unwrap());
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open_home(&location, &execution);
    let bootstrap = fixture.bootstrap_root(&execution);
    let root1 = publish_round(&fixture, &execution, bootstrap.product_branch().clone(), 0);
    drop(bootstrap);
    match scenario.to_str().unwrap() {
        "creation" => {
            let native = create_branch(&fixture, &execution, &root1, "native-fork", true, false);
            let signal = create_branch(&fixture, &execution, &root1, "signal-fork", false, true);
            let both = create_branch(&fixture, &execution, &root1, "both-fork", true, true);
            let moved = publish_round(&fixture, &execution, both, 1);
            let summary = [&native, &signal, &moved]
                .iter()
                .map(|b| {
                    format!(
                        "{}|{}|{}|{}|{}",
                        b.branch_identity().name().as_str(),
                        b.selected_commit().ordinal(),
                        b.reference_generation().get(),
                        b.basis().relational_basis().descriptor().branch_id().0,
                        format!("{:?}", b.basis().signal_basis().branch_id()),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            std::fs::write(markers.join("creation"), summary).unwrap();
            drop((root1, native, signal, moved));
        }
        "retirement" => {
            let sibling = create_branch(&fixture, &execution, &root1, "retired-kill", true, true);
            let moved = publish_round(&fixture, &execution, sibling, 1);
            std::fs::write(
                markers.join("ordinal"),
                moved.selected_commit().ordinal().to_string(),
            )
            .unwrap();
            let report = fixture
                .home
                .world()
                .branch_port()
                .retire_product_branch(&moved)
                .unwrap();
            drop((root1, moved, report));
        }
        "disposal" => {
            let sibling = create_branch(&fixture, &execution, &root1, "disposed-kill", true, true);
            let moved = publish_round(&fixture, &execution, sibling, 1);
            std::fs::write(
                markers.join("ordinal"),
                moved.selected_commit().ordinal().to_string(),
            )
            .unwrap();
            let root2 = publish_round(&fixture, &execution, root1, 2);
            let report = fixture
                .home
                .world()
                .branch_port()
                .retire_product_branch(&moved)
                .unwrap();
            drop((moved, root2));
            let branch_release = fixture
                .home
                .world()
                .recovery_port()
                .release_retired_branch_history(report.retired_head())
                .unwrap();
            assert_eq!(branch_release.reclaimed_commits().len(), 2);
            let interior = fixture
                .home
                .world()
                .recovery_port()
                .retire_changed_history()
                .unwrap();
            assert_eq!(interior.len(), 2);
            drop(report);
        }
        "parent-retired" => {
            let parent_a = create_branch(&fixture, &execution, &root1, "parent-a", true, true);
            let old_incarnation = parent_a.lifecycle_incarnation().ordinal();
            let child_b = create_branch(&fixture, &execution, &parent_a, "child-b", true, true);
            let child_moved = publish_round(&fixture, &execution, child_b, 1);
            let report = fixture
                .home
                .world()
                .branch_port()
                .retire_product_branch(&parent_a)
                .unwrap();
            drop((parent_a, report));
            let reused_a = super::create_branch_with_fork_prefix(
                &fixture,
                &execution,
                &root1,
                "parent-a",
                "parent-a-v2",
                true,
                true,
            );
            std::fs::write(
                markers.join("parent-retired"),
                format!(
                    "{}\n{}\n{}\n",
                    child_moved.selected_commit().ordinal(),
                    old_incarnation,
                    reused_a.lifecycle_incarnation().ordinal(),
                ),
            )
            .unwrap();
            drop((root1, child_moved, reused_a));
        }
        "open-no-effect" => {
            let cancellation = RuntimeWorldCancellationSource::new();
            let prepared = fixture.prepare_publication(
                &execution,
                root1.clone(),
                crate::batches::create(1),
                &cancellation.token(),
            );
            std::fs::write(
                markers.join("open-no-effect"),
                format!(
                    "{}\n{}\n",
                    prepared.attempt_identity().ordinal(),
                    root1.selected_commit().ordinal(),
                ),
            )
            .unwrap();
            // Park with durable Open recorded in Custody and zero Native effect.
            let _keep_alive = (prepared, root1);
            crate::killed_child::announce(&markers.join("ready"), b"open without native effect");
            crate::killed_child::park();
        }
        other => panic!("unknown scenario {other}"),
    }
    crate::killed_child::announce(&markers.join("ready"), b"quiescent after durable operation");
    crate::killed_child::park();
}
