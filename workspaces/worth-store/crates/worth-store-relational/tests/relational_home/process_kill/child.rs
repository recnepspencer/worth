//! Child execution of the process-kill plan; all parks use the existing Store gates.
use super::*;
use std::path::Path;

#[path = "child/cleanup_wal.rs"]
mod cleanup_wal;

/// The child: run the plan for its stop, park, announce, and wait to die.
fn child(stop: Stop, location: &Path, markers: &Path) -> ! {
    let deadline = Instant::now() + killed_child::PARK_DEADLINE;
    let script = script();
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(location, &execution).expect("the child's home opens");
    let (completed, parked) = stop.plan(&script);
    apply(&home, &execution, &script.steps[..completed]);
    if let (Stop::CheckpointAcknowledged(_), Some(profile)) = (stop, child_wal_profile()) {
        cleanup_wal::cover_segments_at_next_checkpoint(&home, profile.wal_segment_bytes());
    }
    if matches!(stop, Stop::CheckpointCarried(_) | Stop::CheckpointReturned) {
        home.checkpoint(&execution).unwrap();
    }
    let acknowledged = workload::artifacts_of(home.runtime());
    killed_child::announce(&markers.join("acknowledged"), &encode(&acknowledged));
    let transitions = super::super::branch_lifecycle::artifacts_of_home(&home);
    killed_child::announce(&markers.join("transitions"), &encode(&transitions));
    let seam = match stop {
        Stop::Commit(seam)
        | Stop::BranchTransition(seam)
        | Stop::Archive(seam)
        | Stop::RequestDeletion(seam)
        | Stop::Delete(seam)
        | Stop::Checkpoint(seam)
        | Stop::CheckpointMembers(seam) => seam,
        Stop::CheckpointAcknowledged(step) => acknowledged_checkpoint(&home, markers, step),
        Stop::CheckpointCarried(seam) => seam,
        Stop::BeforeFirstCheckpoint | Stop::AboveCheckpoint | Stop::CheckpointReturned => {
            let live = acknowledged.len().to_string();
            killed_child::announce(
                &markers.join("transition-live"),
                transitions.len().to_string().as_bytes(),
            );
            killed_child::announce(&markers.join("ready"), live.as_bytes());
            killed_child::park();
        }
    };
    let target = &script.steps[parked.expect("a parked step")];
    std::thread::scope(|scope| {
        let mut gate = home
            .certification_serving()
            .certification_pause_physical_mutation_at(seam);
        let worker = scope.spawn(|| {
            let request = workload::serial_request();
            let execution = ExecutionRequest::serial(&request);
            match target {
                Step::Checkpoint => home
                    .checkpoint(&execution)
                    .map(|_| ())
                    .map_err(|stop| format!("{stop:?}")),
                step if step.is_lifecycle() => step
                    .lifecycle(home.runtime(), &execution)
                    .map_err(|stop| format!("{stop:?}")),
                _ => target
                    .commit(home.runtime(), &execution)
                    .map(|_| ())
                    .map_err(|stop| format!("{stop:?}")),
            }
        });
        if !await_mutation_seam(&gate, &worker, deadline) {
            // A late arrival must not deadlock scope teardown after a failed
            // assertion. Preserve the actual attempt's result in the failure.
            gate.release();
            panic!("{stop:?}: target did not park: {:?}", worker.join());
        }
        if matches!(stop, Stop::Checkpoint(_)) {
            // This small fixture's new members form one batch. Its durable
            // root does not select the checkpoint; the following anchored
            // append is the decision the existing checkpoint court judges.
            // The paused worker owns a clone of the arrived gate. Replacing
            // the registry entry arms its next visit; releasing the old gate
            // cannot release this new one (neither gate unregisters on Drop).
            let anchor = home
                .certification_serving()
                .certification_pause_physical_mutation_at(seam);
            gate.release();
            gate = anchor;
            if !await_mutation_seam(&gate, &worker, deadline) {
                gate.release();
                panic!("{stop:?}: anchor did not park: {:?}", worker.join());
            }
        }
        if let Step::Fork { target } = target {
            assert!(
                home.runtime().branch_identity(target).is_err(),
                "{stop:?}: fork installed before durable publication returned"
            );
        }
        // The Store seam may hold its publication locks. Inspect the live
        // catalog here; exporting old artifacts can perform cold Store reads.
        let live = home
            .runtime()
            .history()
            .immutable_commit_count()
            .to_string();
        killed_child::announce(
            &markers.join("transition-live"),
            super::super::branch_lifecycle::live_count(home.runtime())
                .to_string()
                .as_bytes(),
        );
        killed_child::announce(&markers.join("ready"), live.as_bytes());
        killed_child::park();
    })
}

#[test]
fn relational_home_child() {
    let Some(role) = std::env::var_os(ROLE_ENV) else {
        return;
    };
    let stop = Stop::all()
        .find(|stop| Some(stop.label().as_str()) == role.to_str())
        .expect("a known child role");
    let location = PathBuf::from(std::env::var_os(LOCATION_ENV).expect("child location"));
    let markers = PathBuf::from(std::env::var_os(MARKERS_ENV).expect("child markers"));
    let run = || child(stop, &location, &markers);
    match child_wal_profile() {
        Some(profile) => profile.run(run),
        None => run(),
    }
}

/// These Store gates run only after the home has selected the new directory
/// and acknowledged publication back to Relational's region bindings.
fn acknowledged_checkpoint(
    home: &worth_store_relational::RelationalStoreHome,
    markers: &Path,
    step: worth_store::physical_runtime::production::PhysicalCheckpointStep,
) -> ! {
    let gate = home
        .certification_serving()
        .pause_physical_checkpoint_at(step);
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| home.checkpoint(&execution));
        let deadline = Instant::now() + killed_child::PARK_DEADLINE;
        while !gate.await_arrival() {
            if worker.is_finished() || Instant::now() >= deadline {
                gate.release();
                panic!(
                    "physical checkpoint did not arrive at {step:?}: {:?}",
                    worker.join()
                );
            }
        }
        assert!(!worker.is_finished());
        assert_eq!(super::super::selected_checkpoint::generation(home), 2);
        killed_child::announce(
            &markers.join("transition-live"),
            super::super::branch_lifecycle::live_count(home.runtime())
                .to_string()
                .as_bytes(),
        );
        killed_child::announce(
            &markers.join("ready"),
            home.runtime()
                .history()
                .immutable_commit_count()
                .to_string()
                .as_bytes(),
        );
        killed_child::park();
    })
}
