//! Actual installed Worlds reuse one host ledger; graph allocations are not metered here.
use std::{cell::Cell, num::NonZeroUsize, sync::Arc};
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

use super::product_workflow_support::{self, principal, read_input, ExampleApplication};
use product_workflow_support::adapters::ClockSource;
use product_workflow_support::application::{example_limits, seed_graph};
use product_workflow_support::conditional_contribution::{
    TemporalConditional, TemporalContributionConfiguration,
};
use product_workflow_support::{
    program,
    schema::{TemporalHostSchema, TemporalPrincipalBinding},
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    application_installation::{self, WorthQueryApplicationCheckpoint},
    primary_graph::WorthQueryExecutionLeaseDenial,
    runtime::{
        CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
        ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, LeaseRequest,
    },
};

fn install(
    authority: Arc<ExecutionAuthority>,
    checkpoint: Option<WorthQueryApplicationCheckpoint>,
) -> ExampleApplication {
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = (TemporalContributionConfiguration { clock_source },);
    let limits = example_limits(Some(request().policy)).with_execution_authority(authority);
    let runtime = match checkpoint {
        Some(checkpoint) => application_installation::in_memory_program_from_checkpoint(
            program::validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            configuration,
            limits,
            checkpoint,
        ),
        None => application_installation::in_memory_program(
            program::validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            configuration,
            limits,
            |graph, installed| {
                let binding = installed
                    .principal_binding(TemporalPrincipalBinding::reference())
                    .unwrap();
                seed_graph(graph, &binding, "blocked");
                Ok(())
            },
        ),
    }
    .expect("ordinary installed Temporal schema/program admits the supplied authority");
    let conditional = runtime.conditional::<TemporalConditional>().unwrap();
    ExampleApplication {
        runtime,
        conditional,
        clock_control,
    }
}

fn request() -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 128, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

// Backing is dropped first. This ticket does not account for the installed graph
// or the ledger's own container/allocator overhead.
struct Payload {
    backing: Vec<u8>,
    ticket: ExecutionMemoryReservation,
}

fn payload(
    lease: &ExecutionResourceLease<'_>,
    bytes: usize,
    allocations: &Cell<usize>,
) -> Result<Payload, LeaseDenial> {
    let ticket = lease
        .reserve_memory(u64::try_from(bytes).unwrap())
        .map_err(LeaseDenial::MemoryExhausted)?;
    allocations.set(allocations.get() + 1);
    let mut backing = Vec::new();
    backing.try_reserve_exact(bytes).unwrap();
    assert_eq!(backing.capacity(), bytes);
    backing.resize(bytes, 11);
    Ok(Payload { backing, ticket })
}

fn assert_source(app: &ExampleApplication) {
    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(app, &scope);
    assert_eq!(
        read_input(app, app.runtime.current_world(), &principal, &scope),
        "payload"
    );
}

#[test]
fn execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen() {
    let mut omitted = ExampleApplication::publish("blocked");
    assert!(matches!(
        omitted.runtime.runtime().request_execution_lease(request()),
        Err(WorthQueryExecutionLeaseDenial::Unavailable)
    ));
    omitted.runtime.close_conditional_runtime().unwrap();
    drop(omitted);

    // One construction in this test process; Worlds receive only clones of it.
    let authority = Arc::new(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: Some(128),
        })
        .unwrap(),
    );
    let mut first = install(Arc::clone(&authority), None);
    let mut second = install(Arc::clone(&authority), None);
    assert_source(&first);
    assert_source(&second);
    let allocations = Cell::new(0);
    let retained = {
        let lease = first
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        payload(&lease, 96, &allocations).unwrap()
    };
    {
        let lease = second
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert!(matches!(
            payload(&lease, 33, &allocations),
            Err(LeaseDenial::MemoryExhausted(memory)) if memory == worth_query_host::facade::runtime::MemoryLimitDenial { requested: 33, admitted: 32, level: worth_query_host::facade::runtime::MemoryLimitLevel::Process }
        ));
        assert_eq!(
            allocations.get(),
            1,
            "shared-process refusal precedes payload growth"
        );
    }
    let checkpoint = first
        .runtime
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    first.runtime.close_conditional_runtime().unwrap();
    drop(first);
    second.runtime.close_conditional_runtime().unwrap();
    drop(second);
    assert_eq!(retained.backing, vec![11; 96]);
    assert_eq!(retained.ticket.bytes(), 96);

    let mut reopened = install(Arc::clone(&authority), Some(checkpoint));
    assert!(reopened.runtime.checkpoint_restore_work().is_some());
    assert_source(&reopened);
    let replacement = {
        let lease = reopened
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert!(matches!(
            payload(&lease, 33, &allocations),
            Err(LeaseDenial::MemoryExhausted(memory)) if memory == worth_query_host::facade::runtime::MemoryLimitDenial { requested: 33, admitted: 32, level: worth_query_host::facade::runtime::MemoryLimitLevel::Process }
        ));
        assert_eq!(
            allocations.get(),
            1,
            "old custody survives originating World drop"
        );
        payload(&lease, 32, &allocations).unwrap()
    };
    reopened.runtime.close_conditional_runtime().unwrap();
    drop(reopened);
    assert_eq!(replacement.backing, vec![11; 32]);
    assert_eq!(replacement.ticket.bytes(), 32);
    drop(retained);
    // Replacement still consumes capacity after its own World and lease drop.
    let probe = authority.request_lease(request()).unwrap();
    assert!(matches!(
        probe.reserve_memory(97),
        Err(memory) if memory == worth_query_host::facade::runtime::MemoryLimitDenial { requested: 97, admitted: 96, level: worth_query_host::facade::runtime::MemoryLimitLevel::Process }
    ));
    drop(replacement);
    assert_eq!(probe.reserve_memory(128).unwrap().bytes(), 128);
}
