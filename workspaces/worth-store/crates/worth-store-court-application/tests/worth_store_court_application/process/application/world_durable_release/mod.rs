//! D18 refuses torn publication while acknowledged quiescent and graceful Homes reopen.
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::*;
use worth_store_application::ApplicationHomeCloseOutcome;
mod child;
mod unclean;
const LOCATION: &str = "WORTH_WORLD_RELEASE_HOME";
const MARKERS: &str = "WORTH_WORLD_RELEASE_MARKERS";
const PREFIX: &str = "WORTH_WORLD_RELEASE_PUBLISHED_PREFIX";
const CHILD: &str =
    "process::application::world_durable_release::child::original_world_attempt_child";

fn maximum() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(16).unwrap()
}
fn open(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixture {
    worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    )
    .expect("actual filesystem Home owner readmission")
}

fn close(
    home: worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>,
    execution: &ExecutionRequest<'_, '_>,
) {
    match home.close(execution) {
        Ok(ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_), ..
        }) => {}
        Err(refused) => panic!("World close refused: {:?}", refused.reason()),
        Ok(ApplicationHomeCloseOutcome::CloseRefused { reason, .. }) => {
            panic!("Native close refused: {reason:?}")
        }
        Ok(ApplicationHomeCloseOutcome::Closed {
            checkpoint: Err(cause),
            ..
        }) => {
            panic!("the original coupled checkpoint must complete before physical close: {cause:?}")
        }
    }
}
