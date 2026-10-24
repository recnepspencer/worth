//! Recovery interrupted: a home whose last commit was killed with a durable
//! WAL record (so recovery has a publication to roll forward) is opened by a
//! child whose Store recovery parks at one of its own stages and is killed
//! there. The next recovery, and a second and third after it, reach the same
//! history and states, and the home then finishes the script.

use std::fs;
use std::path::PathBuf;

use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint as Seam;
use worth_store::physical_runtime::{
    PhysicalRecoveryProcessYieldpoint, PhysicalRecoveryYieldpointStage as Stage,
};
use worth_store_relational::{RelationalStoreHome, RelationalStoreHomeRequest};

use super::crash_oracle::{assert_continues, assert_recovered, InFlight};
use super::durable_run::{assert_holds, open, recorded_schema};
use super::killed_child::{self, ChildEnd};
use super::process_kill::{self, Stop, SEED};
use super::schema_transition;
use super::workload;

const STAGE_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_STAGE";
const LOCATION_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_LOCATION";
const MARKERS_ENV: &str = "WORTH_RELATIONAL_HOME_RECOVERY_MARKERS";
const RECOVERY_CHILD: &str = "recovery_interruption::recovery_interruption_child";

const STAGES: [Stage; 9] = [
    Stage::StagingMaterialization,
    Stage::StagingSynchronization,
    Stage::CandidateMaterialization,
    Stage::CandidateSynchronization,
    Stage::RootProtocolReplacement,
    Stage::RecordNamespaceSynchronization,
    Stage::FreshReopenCurrentSelector,
    Stage::FreshReopenExactBinding,
    Stage::CleanupRemoval,
];

#[test]
fn a_recovery_killed_at_each_of_its_stages_recovers_the_same_home_three_times() {
    let script = process_kill::script();
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let mut interrupted = 0;
    for stage in STAGES {
        let case = format!("seed {SEED:#x} recovery killed at {}", stage.label());
        let killed = process_kill::launch(Stop::Commit(Seam::AfterWalDurability));
        let markers = tempfile::tempdir().unwrap();
        let label = PathBuf::from(stage.label());
        let end = killed_child::run_and_kill(
            RECOVERY_CHILD,
            &[
                (STAGE_ENV, &label),
                (LOCATION_ENV, &killed.location),
                (MARKERS_ENV, markers.path()),
            ],
            &markers.path().join("reached"),
            &markers.path().join("finished"),
        );
        interrupted += usize::from(end == ChildEnd::KilledAtPark);

        let home = open(&killed.location, &execution)
            .unwrap_or_else(|refusal| panic!("{case}: recovery refused: {refusal:?}"));
        let commits = assert_recovered(
            &home,
            &script,
            &killed.acknowledged,
            &InFlight::Durable,
            &case,
        );
        let recovered = workload::artifacts_of(home.runtime());
        home.close();
        for attempt in ["second", "third"] {
            let again = open(&killed.location, &execution)
                .unwrap_or_else(|refusal| panic!("{case}: {attempt} recovery: {refusal:?}"));
            assert_holds(&again, &script, &recovered);
            if attempt == "third" {
                assert_continues(&killed.location, again, &script, commits, &execution, &case);
            } else {
                again.close();
            }
        }
    }
    assert!(interrupted > 0, "no recovery stage was reached");
    eprintln!(
        "recovery interrupted at {interrupted} of {} stages",
        STAGES.len()
    );
}

#[test]
fn recovery_interruption_child() {
    let Some(label) = std::env::var_os(STAGE_ENV) else {
        return;
    };
    let stage = Stage::from_label(label.to_str().unwrap()).expect("a recovery stage label");
    let location = PathBuf::from(std::env::var_os(LOCATION_ENV).unwrap());
    let markers = PathBuf::from(std::env::var_os(MARKERS_ENV).unwrap());
    let yieldpoint = PhysicalRecoveryProcessYieldpoint::new(
        stage,
        markers.join("reached"),
        markers.join("release"),
        markers.join("cancel"),
    );
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let opened = RelationalStoreHome::certification_open_with_recovery_yieldpoint(
        RelationalStoreHomeRequest::new(
            &location,
            workload::runtime_builder(schema_transition::SOURCE),
        ),
        yieldpoint,
        &execution,
    );
    // The killed commit is the script's durable schema transition, so the
    // recovered home serves its target: an open that declared the source
    // completed recovery and was refused for it, and serves once it declares
    // the target.
    let home = match opened {
        Err(refusal) if recorded_schema(&refusal) == Some(schema_transition::TARGET) => {
            open(&location, &execution)
        }
        opened => opened,
    };
    home.expect("a recovery that never parks completes").close();
    fs::write(markers.join("finished"), b"1").unwrap();
}
