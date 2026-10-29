//! Installed Worlds return bootstrap admission and share one host ledger for retained custody.
pub use std::{cell::Cell, num::NonZeroUsize, sync::Arc};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

pub use super::product_workflow_support::{self, principal, read_input, ExampleApplication};
pub use product_workflow_support::adapters::ClockSource;
pub use product_workflow_support::application::{example_limits, seed_graph};
pub use product_workflow_support::conditional_contribution::{
    TemporalConditional, TemporalContributionConfiguration,
};
pub use product_workflow_support::{
    program,
    schema::{TemporalHostSchema, TemporalPrincipalBinding},
};
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::{
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
pub const AUTHORITY_BYTES: usize = 8 * 1024 * 1024;
pub const RETAINED_BYTES: usize = 96;

pub fn install(
    authority: Arc<ExecutionAuthority>,
    checkpoint: Option<WorthQueryApplicationCheckpoint>,
) -> ExampleApplication {
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = (TemporalContributionConfiguration::new(clock_source),);
    // Installation performs the example's checked bootstrap, not one ordinary
    // read. Keep this proof's exact memory bound and the example's declared work.
    let work = product_workflow_support::application::product_world_resources()
        .execution_policy()
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

pub fn request() -> LeaseRequest {
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
pub struct Payload {
    pub backing: Vec<u8>,
    pub ticket: ExecutionMemoryReservation,
}

pub fn payload(
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

pub fn assert_source(app: &ExampleApplication) {
    let scope = product_workflow_support::adapters::request_scope();
    let principal = principal(app, &scope);
    assert_eq!(
        read_input(app, app.runtime.current_world(), &principal, &scope),
        "payload"
    );
}
