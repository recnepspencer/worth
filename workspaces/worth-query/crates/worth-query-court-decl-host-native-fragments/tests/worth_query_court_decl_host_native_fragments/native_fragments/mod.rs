//! One-of courts cross installed contracts, actual Native state and Query phases.
use worth_execution::ExecutionAllocationPolicy::SystemAllocation;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_host::facade::domain::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_decl::fragment_auth_fixture::*;
use worth_query_testkit_decl::fragment_fixture::*;
mod authentication;
mod candidate;
mod inspection;
mod read;
mod resources;
mod world;
mod write;
pub(crate) use world::{Mode, World};

fn work<Output>(run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> Output) -> Output {
    let mut admit = |_, _| Ok(());
    run(&mut admit)
}
pub(crate) fn measured() -> Measurements {
    Measurements {
        components: [17, 23, 31],
        interval: Interval::checked(41, 53).unwrap(),
    }
}

mod extensions;

mod relation_compile_fail;

mod relation_admission;
