//! The actual World owner participates in Native checkpoint publication.
use worth_execution::ExecutionRequest;
use worth_store_testkit_application_oracle::facade::{home_opening, world_home};

pub(super) fn open(
    location: &std::path::Path,
    builder: worth_relational::facade::runtime::RelationalRuntimeBuilder,
    initialize_world: bool,
    execution: &ExecutionRequest<'_, '_>,
) -> world_home::WorldHomeFixture {
    let fixture = world_home::open(home_opening::request(location, builder), execution).unwrap();
    if initialize_world {
        drop(fixture.bootstrap_root(execution));
    }
    fixture
}

pub(super) fn workload(
    location: &std::path::Path,
    initialize_world: bool,
    execution: &ExecutionRequest<'_, '_>,
) -> world_home::WorldHomeFixture {
    open(
        location,
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        initialize_world,
        execution,
    )
}
