//! Native script journeys retain the actual Home World and Signal owners.
use super::*;
use worth_store_application::WorldReadmittedApplicationHome;
use worth_store_testkit_application_oracle::facade::{home_opening, world_home};

pub(crate) fn open(
    location: &Path,
    schema: SchemaVersionId,
    initialize_world: bool,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<world_home::WorldHomeFixture, world_home::WorldHomeFixtureRefusal> {
    let fixture = world_home::open(
        home_opening::request(location, crate::workload::runtime_builder(schema)),
        execution,
    )?;
    if initialize_world {
        drop(fixture.bootstrap_root(execution));
    }
    Ok(fixture)
}

pub(crate) fn apply(
    home: &WorldReadmittedApplicationHome<(), (), (), (), ()>,
    execution: &ExecutionRequest<'_, '_>,
    steps: &[Step],
) -> usize {
    super::apply_steps(home.relational().runtime(), execution, steps, || {
        home.checkpoint(execution)
            .expect("actual World-owned Home checkpoint");
    })
}

pub(crate) fn assert_holds(
    execution: &ExecutionRequest<'_, '_>,
    home: &WorldReadmittedApplicationHome<(), (), (), (), ()>,
    script: &Script,
    acknowledged: &[Vec<u8>],
) {
    super::assert_holds_native(execution, home.relational().runtime(), script, acknowledged);
    super::assert_lifecycle_state(
        home.relational().runtime(),
        crate::branch_lifecycle::artifacts_of_world(home),
        script,
        acknowledged.len(),
        "held",
    );
}

pub(crate) fn close(fixture: world_home::WorldHomeFixture, execution: &ExecutionRequest<'_, '_>) {
    let outcome = fixture
        .home
        .close(execution)
        .unwrap_or_else(|refused| panic!("actual World Home close: {:?}", refused.reason()));
    assert!(
        matches!(
            outcome,
            worth_store_application::ApplicationHomeCloseOutcome::Closed { .. }
        ),
        "graceful World Home close must retire its actual owners"
    );
}
