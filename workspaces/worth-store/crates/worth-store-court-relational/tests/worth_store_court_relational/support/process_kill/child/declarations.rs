//! Child execution of the process-kill plan; all parks use the existing Store gates.
pub(crate) use std::path::Path;
pub(crate) use {
    crate::durable_run::apply, crate::durable_run::open, crate::killed_child,
    crate::process_kill::await_mutation_seam, crate::process_kill::child_wal_profile,
    crate::process_kill::encode, crate::process_kill::script, crate::process_kill::Stop,
    crate::process_kill::LOCATION_ENV, crate::process_kill::MARKERS_ENV,
    crate::process_kill::ROLE_ENV, crate::workload, crate::workload::Step, std::path::PathBuf,
    std::time::Instant, worth_execution::ExecutionRequest,
    worth_store::physical_runtime::production::PhysicalCheckpointStep,
};

/// The child: run the plan for its stop, park, announce, and wait to die.
pub(crate) fn child(stop: Stop, location: &Path, markers: &Path) -> ! {
    let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
    let script = script();
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = open(location, &execution).expect("the child's home opens");
    let (completed, parked) = stop.plan(&script);
    apply(&home, &execution, &script.steps[..completed]);
    if let (crate::process_kill::Stop::CheckpointAcknowledged(_), Some(profile)) =
        (stop, child_wal_profile())
    {
        crate::process_kill::child::cleanup_wal::cover_segments_at_next_checkpoint(
            &home,
            profile.wal_segment_bytes(),
        );
    }
    if matches!(
        stop,
        crate::process_kill::Stop::CheckpointCarried(_)
            | crate::process_kill::Stop::CheckpointReturned
    ) {
        home.checkpoint(&execution).unwrap();
    }
    let acknowledged = crate::workload::artifacts_of(home.runtime());
    crate::killed_child::announce(&markers.join("acknowledged"), &encode(&acknowledged));
    let transitions = crate::branch_lifecycle::artifacts_of_home(&home);
    crate::killed_child::announce(&markers.join("transitions"), &encode(&transitions));
    let seam = match stop {
        crate::process_kill::Stop::Commit(seam)
        | crate::process_kill::Stop::BranchTransition(seam)
        | crate::process_kill::Stop::Archive(seam)
        | crate::process_kill::Stop::RequestDeletion(seam)
        | crate::process_kill::Stop::Delete(seam)
        | crate::process_kill::Stop::Checkpoint(seam)
        | crate::process_kill::Stop::CheckpointMembers(seam) => seam,
        crate::process_kill::Stop::CheckpointAcknowledged(step) => {
            acknowledged_checkpoint(&home, markers, step)
        }
        crate::process_kill::Stop::CheckpointCarried(seam) => seam,
        crate::process_kill::Stop::BeforeFirstCheckpoint
        | crate::process_kill::Stop::AboveCheckpoint
        | crate::process_kill::Stop::CheckpointReturned => {
            let live = acknowledged.len().to_string();
            crate::killed_child::announce(
                &markers.join("transition-live"),
                transitions.len().to_string().as_bytes(),
            );
            crate::killed_child::announce(&markers.join("ready"), live.as_bytes());
            crate::killed_child::park();
        }
    };
    let target = &script.steps[parked.expect("a parked step")];
    std::thread::scope(|scope| {
        let mut gate = home
            .certification_serving()
            .certification_pause_physical_mutation_at(seam);
        let worker = scope.spawn(|| {
            let request = crate::workload::serial_request();
            let execution = worth_execution::ExecutionRequest::serial(&request);
            match target {
                crate::workload::Step::Checkpoint => home
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
        if matches!(stop, crate::process_kill::Stop::Checkpoint(_)) {
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
        if let crate::workload::Step::Fork { target } = target {
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
        crate::killed_child::announce(
            &markers.join("transition-live"),
            crate::branch_lifecycle::live_count(home.runtime())
                .to_string()
                .as_bytes(),
        );
        crate::killed_child::announce(&markers.join("ready"), live.as_bytes());
        crate::killed_child::park();
    })
}

/// These Store gates run only after the home has selected the new directory
/// and acknowledged publication back to Relational's region bindings.
pub(crate) fn acknowledged_checkpoint(
    home: &worth_store_relational::RelationalStoreHome,
    markers: &Path,
    step: worth_store::physical_runtime::production::PhysicalCheckpointStep,
) -> ! {
    let gate = home
        .certification_serving()
        .pause_physical_checkpoint_at(step);
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| home.checkpoint(&execution));
        let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
        while !gate.await_arrival() {
            if worker.is_finished() || std::time::Instant::now() >= deadline {
                gate.release();
                panic!(
                    "physical checkpoint did not arrive at {step:?}: {:?}",
                    worker.join()
                );
            }
        }
        assert!(!worker.is_finished());
        assert_eq!(crate::selected_checkpoint::generation(home), 2);
        crate::killed_child::announce(
            &markers.join("transition-live"),
            crate::branch_lifecycle::live_count(home.runtime())
                .to_string()
                .as_bytes(),
        );
        crate::killed_child::announce(
            &markers.join("ready"),
            home.runtime()
                .history()
                .immutable_commit_count()
                .to_string()
                .as_bytes(),
        );
        crate::killed_child::park();
    })
}
