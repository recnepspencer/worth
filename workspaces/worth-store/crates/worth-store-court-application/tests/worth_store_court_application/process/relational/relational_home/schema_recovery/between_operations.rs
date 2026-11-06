//! A real kill between completed script operations preserves the selected tail.
use super::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PLAN: &str = "WORTH_SCHEMA_JOURNEY_PLAN";
const CHILD: &str = "process::relational::relational_home::schema_recovery::between_operations::schema_journey_child";

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) enum ScriptPhase {
    Prefix,
    Suffix,
}

#[derive(Serialize, Deserialize)]
struct ChildPlan {
    location: PathBuf,
    checkpoint_every: usize,
    prefix: usize,
    phase: ScriptPhase,
    acknowledged: Vec<Vec<u8>>,
    announced: PathBuf,
}

pub(super) fn run(
    location: &Path,
    checkpoint_every: usize,
    prefix: usize,
    phase: ScriptPhase,
    acknowledged: &[Vec<u8>],
    reports: &Path,
) -> Vec<Vec<u8>> {
    let plan_path = reports.join("schema-journey-plan.json");
    let announced = reports.join("schema-journey-announced.json");
    // Both phases retain their actual receipt bytes; reuse no prior announcement.
    if announced.exists() {
        std::fs::remove_file(&announced).unwrap();
    }
    let plan = ChildPlan {
        location: location.into(),
        checkpoint_every,
        prefix,
        phase,
        acknowledged: acknowledged.to_vec(),
        announced: announced.clone(),
    };
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let ended = crate::killed_child::run_and_kill(
        CHILD,
        &[(PLAN, &plan_path)],
        &announced,
        &reports.join("schema-journey-finished"),
    );
    assert_eq!(ended, crate::killed_child::ChildEnd::KilledAtPark);
    serde_json::from_slice(&std::fs::read(announced).unwrap()).unwrap()
}

#[test]
fn schema_journey_child() {
    let Some(path) = std::env::var_os(PLAN) else {
        return;
    };
    let plan: ChildPlan = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let script = crate::workload::Script::generate(SEED, COMMITS, plan.checkpoint_every);
    let serial = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let (schema, initialize) = match plan.phase {
        ScriptPhase::Prefix => (SOURCE, true),
        ScriptPhase::Suffix => (served_schema(steps_before(&script, plan.prefix)), false),
    };
    let fixture =
        crate::durable_run::world::open(&plan.location, schema, initialize, &execution).unwrap();
    let steps = match plan.phase {
        ScriptPhase::Prefix => steps_before(&script, plan.prefix),
        ScriptPhase::Suffix => {
            crate::durable_run::world::assert_holds(
                &execution,
                &fixture.home,
                &script,
                &plan.acknowledged,
            );
            crate::durable_run::steps_after(&script, plan.prefix)
        }
    };
    let applied = crate::durable_run::world::apply(&fixture.home, &execution, steps);
    let expected = match plan.phase {
        ScriptPhase::Prefix => plan.prefix,
        ScriptPhase::Suffix => COMMITS - plan.prefix,
    };
    assert_eq!(applied, expected);
    let actual = crate::workload::artifacts_of(&execution, fixture.home.relational().runtime());
    crate::durable_run::world::assert_holds(&execution, &fixture.home, &script, &actual);
    crate::killed_child::announce(&plan.announced, &serde_json::to_vec(&actual).unwrap());
    // Actual owners remain live until the parent kills this process.
    crate::killed_child::park();
}
