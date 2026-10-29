pub use {std::io::Write, std::path::Path, std::process::ExitStatus};

pub use {worth_store::physical_runtime::PhysicalStoreClosePhase};

pub const PHASE_ENV: &str = "WORTH_STORE_C5_CLOSE_DEATH_PHASE";

pub fn writer(root: &Path) {
    let serving = crate::physical_record_journeys::serving_from_initialization(root);
    let plan = serving.close_plan();
    let observation = plan.observation();
    let selected = selected_phase();
    let watcher = std::thread::spawn(move || exit_when_reached(observation, selected));
    let _ = plan.execute();
    watcher.join().unwrap();
    panic!("selected close phase did not terminate the process");
}

pub fn reopener(root: &Path) {
    let serving = crate::physical_record_journeys::serving_from_open(root);
    println!("C5_CLOSE_REOPENED {:?}", serving.store_identity());
    let _ = serving.close_plan().execute();
}

pub fn kill_writer_at(root: &Path, phase: PhysicalStoreClosePhase) -> String {
    let output = crate::physical_record_journeys::child_process::invocation::child_command("close_phase_writer", root)
        .env(PHASE_ENV, format!("{phase:?}"))
        .output()
        .unwrap();
    assert_expected_death(output.status);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout
            .lines()
            .any(|line| line == format!("C5_CLOSE_PHASE {phase:?}")),
        "writer did not reach {phase:?}; stdout:\n{stdout}"
    );
    stdout
}

pub fn selected_phase() -> PhysicalStoreClosePhase {
    let selected = std::env::var(PHASE_ENV).expect("close death phase");
    [
        worth_store::physical_runtime::PhysicalStoreClosePhase::CheckpointDrained,
        worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped,
        worth_store::physical_runtime::PhysicalStoreClosePhase::SafeCancellationComplete,
        worth_store::physical_runtime::PhysicalStoreClosePhase::DispatchSettlementComplete,
        worth_store::physical_runtime::PhysicalStoreClosePhase::SignalDisposed,
        worth_store::physical_runtime::PhysicalStoreClosePhase::ResidencyClosed,
        worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased,
    ]
    .into_iter()
    .find(|phase| selected == phase_name(*phase))
    .expect("known close death phase")
}

pub fn exit_when_reached(
    observation: worth_store::physical_runtime::PhysicalStoreCloseObservation,
    selected: PhysicalStoreClosePhase,
) {
    while !observation.reached(selected) {
        std::thread::yield_now();
    }
    println!("C5_CLOSE_PHASE {selected:?}");
    std::io::stdout().flush().unwrap();
    std::process::exit(86);
}

pub const fn phase_name(phase: PhysicalStoreClosePhase) -> &'static str {
    match phase {
        worth_store::physical_runtime::PhysicalStoreClosePhase::CheckpointDrained => "CheckpointDrained",
        worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped => "AdmissionStopped",
        worth_store::physical_runtime::PhysicalStoreClosePhase::SafeCancellationComplete => "SafeCancellationComplete",
        worth_store::physical_runtime::PhysicalStoreClosePhase::DispatchSettlementComplete => "DispatchSettlementComplete",
        worth_store::physical_runtime::PhysicalStoreClosePhase::SignalDisposed => "SignalDisposed",
        worth_store::physical_runtime::PhysicalStoreClosePhase::ResidencyClosed => "ResidencyClosed",
        worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased => "MediaReleased",
    }
}

pub fn assert_expected_death(status: ExitStatus) {
    assert_eq!(
        status.code(),
        Some(86),
        "close writer must die at its selected production phase"
    );
}