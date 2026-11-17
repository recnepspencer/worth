//! Two independent Directory Homes and a reopened root share one process service.
// The mounted test inherits its crate root's limit; this example is test=false.
#![cfg_attr(not(test), recursion_limit = "256")]
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::*,
    application_entry::*,
    application_installation as installation, primary_graph,
    runtime::{
        ConstructionDenial, DeterminismContract, ExecutionAllocationPolicy, ExecutionAuthority,
        ExecutionAuthorityConfig, ExecutionPosture, ExecutionRequestPolicy,
    },
};
use worth_query_topology_entry::*;

#[path = "required_feature/program.rs"]
mod program;
// The required_feature example owns and lints this setup; this example uses a part.
#[path = "shared_execution/home.rs"]
mod home;
#[allow(dead_code)]
#[path = "required_feature/installation.rs"]
mod setup;
use program::{ExampleProgram, ExampleSchema, FinalConnection, Root};
type Application =
    installation::WorthQueryProgramApplicationRuntime<ExampleSchema, ExampleProgram<false>>;

#[cfg(not(test))]
fn main() {
    worth_execution::run_with_execution_stack(run);
}

pub(crate) fn run() {
    // Resolve immutable build input before constructing any worker threads.
    let build =
        installation::ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID"))
            .unwrap();
    let workers = ExecutionAuthorityConfig::available_worker_count();
    let service = Arc::new(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig::for_workers(workers)).unwrap(),
    );
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        workers,
    );
    let limits = || setup::limits_for_policy(policy).with_execution_authority(Arc::clone(&service));
    let directory = std::env::temp_dir().join(format!(
        "worth-shared-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let first_directory = directory.join("first");
    let second_directory = directory.join("second");
    let first = home::open(&first_directory, build, limits());
    let second = home::open(&second_directory, build, limits());
    settle_and_check(first.runtime(), 3);
    settle_and_check(second.runtime(), 3);
    home::close(first);
    let reopened = home::open(&first_directory, build, limits());
    let application = reopened.runtime();
    settle_and_check(application, 3);
    {
        let (scope, principal) = setup::authenticate(application);
        let request = application.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let outcome = request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: setup::length(9),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&91_u64)
            .execute_performed::<ExampleProgram<false>, Root>(
                application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        match outcome {
            WorthQueryApplicationPerformedMutationOutcome::Performed(_) => {}
            WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
                panic!("source adjustment was not performed: {outcome:?}");
            }
            WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { .. } => {
                panic!("source adjustment requires cleanup");
            }
            WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(_) => {
                panic!("source adjustment product is unpublished");
            }
            WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(_) => {
                panic!("source adjustment required output was denied");
            }
            WorthQueryApplicationPerformedMutationOutcome::Blocked(_) => {
                panic!("source adjustment is blocked");
            }
        }
    }
    settle_and_check(application, 11);
    // A different installed root still owns independent model state.
    settle_and_check(second.runtime(), 3);
    home::close(reopened);
    home::close(second);
    drop(service);
    assert!(matches!(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig::for_workers(workers)),
        Err(ConstructionDenial::AlreadyConstructed)
    ));
}

fn settle_and_check(application: &Application, expected: u64) {
    let (scope, principal) = setup::authenticate(application);
    let request = application.request(&principal, &scope);
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<ExampleProgram<false>, FinalConnection>(application)
        .unwrap();
    loop {
        if let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.advance(&request).unwrap()
        {
            let output = request
                .query(PlanarOutputRead {
                    body_key: "final:anchor-a".into(),
                })
                .execute()
                .unwrap();
            assert_eq!(output.rows().len(), 1);
            assert_eq!(output.rows()[0].value, setup::length(expected));
            drop(settled);
            return;
        }
    }
}
