//! Installed Worlds return bootstrap admission and share one host ledger for retained custody.
use std::{cell::Cell, num::NonZeroUsize, sync::Arc};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
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
        MemoryLimitDenial, MemoryLimitLevel,
    },
};

// Headroom for bootstrap and charged native reads; the ledger proof below fills
// this capacity and does not depend on its absolute size.
const AUTHORITY_BYTES: usize = 64 * 1024;
const RETAINED_BYTES: usize = 96;

fn install(
    authority: Arc<ExecutionAuthority>,
    checkpoint: Option<WorthQueryApplicationCheckpoint>,
) -> ExampleApplication {
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = (TemporalContributionConfiguration { clock_source },);
    // Installation performs the example's checked bootstrap, not one ordinary
    // read. Keep this proof's exact memory bound and the example's declared work.
    let work = product_workflow_support::application::product_world_resources()
        .execution_policy()
        .budget()
        .work_ceiling();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, Some(AUTHORITY_BYTES as u64), work),
    );
    let limits = example_limits(Some(policy)).with_execution_authority(authority);
    let runtime = match checkpoint {
        Some(checkpoint) => application_installation::in_memory_program_from_checkpoint(
            program::validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            configuration,
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            limits,
            checkpoint,
        ),
        None => application_installation::in_memory_program(
            program::validated_program(),
            TemporalHostSchema::declaration().unwrap(),
            configuration,
            ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
            limits,
            |_phase, graph, installed| {
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
            ExecutionBudget::new(
                NonZeroUsize::new(1).unwrap(),
                Some(AUTHORITY_BYTES as u64),
                None,
            ),
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
    // The process-memory ceiling belongs to this proof's composition root.
    // Other tests in this binary construct the shared placement authority.
    const CHILD: &str = "WORTH_QUERY_MEMORY_LEDGER_PROOF_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = format!(
            "{}::execution_memory_is_shared_by_installed_worlds_and_reselected_on_checkpoint_reopen",
            module_path!().split_once("::").unwrap().1
        );
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        return;
    }
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
            charged_memory_bytes: Some(AUTHORITY_BYTES as u64),
        })
        .unwrap(),
    );
    let mut first = install(Arc::clone(&authority), None);
    let mut second = install(Arc::clone(&authority), None);
    assert_source(&first);
    assert_source(&second);
    {
        let lease = first
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert_eq!(
            lease
                .reserve_memory(AUTHORITY_BYTES as u64)
                .unwrap()
                .bytes(),
            AUTHORITY_BYTES as u64,
            "both installations return all bootstrap admission"
        );
    }
    let allocations = Cell::new(0);
    let retained = {
        let lease = first
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        payload(&lease, RETAINED_BYTES, &allocations).unwrap()
    };
    {
        let lease = second
            .runtime
            .runtime()
            .request_execution_lease(request())
            .unwrap();
        assert!(matches!(
            payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES + 1, &allocations),
            Err(LeaseDenial::MemoryExhausted(memory)) if memory == (MemoryLimitDenial::Capacity {
                requested: (AUTHORITY_BYTES - RETAINED_BYTES + 1) as u64,
                admitted: (AUTHORITY_BYTES - RETAINED_BYTES) as u64,
                level: MemoryLimitLevel::Process,
            })
        ));
        assert_eq!(
            allocations.get(),
            1,
            "shared-process refusal precedes payload growth"
        );
    }
    let checkpoint = first
        .runtime
        .with_host_advancement(|host_phase| {
            first
                .runtime
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    first.runtime.close_conditional_runtime().unwrap();
    drop(first);
    second.runtime.close_conditional_runtime().unwrap();
    drop(second);
    assert_eq!(retained.backing, vec![11; RETAINED_BYTES]);
    assert_eq!(retained.ticket.bytes(), RETAINED_BYTES as u64);

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
            payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES + 1, &allocations),
            Err(LeaseDenial::MemoryExhausted(memory)) if memory == (MemoryLimitDenial::Capacity {
                requested: (AUTHORITY_BYTES - RETAINED_BYTES + 1) as u64,
                admitted: (AUTHORITY_BYTES - RETAINED_BYTES) as u64,
                level: MemoryLimitLevel::Process,
            })
        ));
        assert_eq!(
            allocations.get(),
            1,
            "old custody survives originating World drop"
        );
        payload(&lease, AUTHORITY_BYTES - RETAINED_BYTES, &allocations).unwrap()
    };
    reopened.runtime.close_conditional_runtime().unwrap();
    drop(reopened);
    assert_eq!(
        replacement.backing,
        vec![11; AUTHORITY_BYTES - RETAINED_BYTES]
    );
    assert_eq!(
        replacement.ticket.bytes(),
        (AUTHORITY_BYTES - RETAINED_BYTES) as u64
    );
    drop(retained);
    // Replacement still consumes capacity after its own World and lease drop.
    let probe = authority.request_lease(request()).unwrap();
    assert!(matches!(
        probe.reserve_memory((RETAINED_BYTES + 1) as u64),
        Err(memory) if memory == (MemoryLimitDenial::Capacity {
                requested: (RETAINED_BYTES + 1) as u64,
                admitted: RETAINED_BYTES as u64,
                level: MemoryLimitLevel::Process,
            })
    ));
    drop(replacement);
    assert_eq!(
        probe
            .reserve_memory(AUTHORITY_BYTES as u64)
            .unwrap()
            .bytes(),
        AUTHORITY_BYTES as u64
    );
}
