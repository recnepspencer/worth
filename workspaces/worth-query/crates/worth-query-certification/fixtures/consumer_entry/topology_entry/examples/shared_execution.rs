//! Two installed applications and a restored root share one process service.
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
#[path = "required_feature/installation.rs"]
mod setup;
use program::{ExampleProgram, ExampleSchema, FinalConnection, Root};
type Application =
    installation::WorthQueryProgramApplicationRuntime<ExampleSchema, ExampleProgram<false>>;

#[cfg(not(test))]
fn main() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
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
    let mut first =
        setup::install_configured::<false>(setup::configuration(), build, limits(), || {}).unwrap();
    let mut second =
        setup::install_configured::<false>(setup::configuration(), build, limits(), || {}).unwrap();
    settle_and_check(&first, 3);
    settle_and_check(&second, 3);
    let checkpoint = first
        .with_host_advancement(|phase| {
            first.capture_application_checkpoint(
                &phase,
                installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
        })
        .unwrap()
        .unwrap();
    close(&mut first);
    drop(first);

    let program = ApplicationProgramAuthoring::<ExampleSchema, ExampleProgram<false>>::begin()
        .validated_program()
        .unwrap();
    let mut reopened = installation::in_memory_program_from_checkpoint(
        program,
        ExampleSchema::declaration().unwrap(),
        (setup::configuration(),),
        build,
        limits(),
        checkpoint,
    )
    .unwrap();
    settle_and_check(&reopened, 3);
    {
        let (scope, principal) = setup::authenticate(&reopened);
        let request = reopened.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: setup::length(9),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&91_u64)
            .execute_performed::<ExampleProgram<false>, Root>(
                &reopened,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    settle_and_check(&reopened, 11);
    // A different installed root still owns independent model state.
    settle_and_check(&second, 3);
    close(&mut reopened);
    close(&mut second);
    drop((reopened, second));
    drop(service);
    assert!(matches!(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig::for_workers(workers)),
        Err(ConstructionDenial::AlreadyConstructed)
    ));
}

fn close(application: &mut Application) {
    assert!(application.close_live_delivery().owner_terminal());
    application.close_conditional_runtime().unwrap();
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
