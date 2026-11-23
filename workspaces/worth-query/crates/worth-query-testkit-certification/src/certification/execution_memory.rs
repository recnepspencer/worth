//! Installed Worlds return bootstrap admission and share one host ledger for retained custody.
//! Re-expressed through the public `WorthQueryApplicationHome` directory open/close/reopen path.
pub use std::{
    cell::Cell,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;

pub use super::product_workflow_support::{self, principal, read_input, ExampleApplication};
pub use product_workflow_support::adapters::ClockSource;
pub use product_workflow_support::application::{
    admit_identity_adapter, example_limits, product_world_resources, seed_graph,
};
pub use product_workflow_support::conditional_contribution::{
    TemporalConditional, TemporalContributionConfiguration,
};
pub use product_workflow_support::{
    application_entry::TemporalIntentRead,
    program::{self, TemporalExampleProgram},
    schema::{TemporalHostSchema, TemporalPrincipalBinding},
};
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::{self, WorthQueryProgramApplicationRuntime},
    primary_graph::WorthQueryExecutionLeaseDenial,
    runtime::{
        CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
        ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, LeaseRequest,
        MemoryLimitDenial, MemoryLimitLevel,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

// Headroom for bootstrap and charged native reads; the ledger proof below fills
// this capacity and does not depend on its absolute size.
pub const AUTHORITY_BYTES: usize = 8 * 1024 * 1024;
pub const RETAINED_BYTES: usize = 96;

pub struct TempHomeDir(PathBuf);

impl TempHomeDir {
    pub fn new(tag: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(1);
        let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "worth-c2-5-exec-mem-{tag}-{}-{ordinal}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create temp home directory");
        Self(path)
    }

    pub fn path(&self) -> PathBuf {
        self.0.clone()
    }
}

impl Drop for TempHomeDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn shared_limits(
    authority: Arc<ExecutionAuthority>,
) -> application_installation::WorthQueryApplicationLimits {
    let work = product_world_resources().execution_policy().work_ceiling();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, Some(AUTHORITY_BYTES as u64), work),
    );
    example_limits(Some(policy)).with_execution_authority(authority)
}

pub fn open_directory_home(
    location: PathBuf,
    authority: Arc<ExecutionAuthority>,
    bootstrap_execution: worth_execution::ExecutionRequest<'_, '_>,
    initialized: Arc<AtomicUsize>,
) -> WorthQueryApplicationHome<TemporalHostSchema, TemporalExampleProgram> {
    let (clock_source, _clock_control) = ClockSource::due();
    let configuration = (TemporalContributionConfiguration::new(clock_source),);
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        program::validated_program(),
        application_installation::WorthQueryApplicationProgramRoster::new(),
        TemporalHostSchema::declaration().unwrap(),
        configuration,
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        shared_limits(authority),
        move |_phase, graph, installed| {
            initialized.fetch_add(1, Ordering::SeqCst);
            let binding = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &binding, "blocked");
            Ok(())
        },
    );
    with_bootstrap_advancement_from_request(bootstrap_execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .expect("bootstrap execution opens")
    .expect("directory Home opens with shared execution authority")
}

pub fn install_in_memory(authority: Arc<ExecutionAuthority>) -> ExampleApplication {
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = (TemporalContributionConfiguration::new(clock_source),);
    let runtime = application_installation::in_memory_program(
        program::validated_program(),
        TemporalHostSchema::declaration().unwrap(),
        configuration,
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        shared_limits(authority),
        |_phase, graph, installed| {
            let binding = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &binding, "blocked");
            Ok(())
        },
    )
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

pub fn bootstrap_serial_request() -> worth_execution::SerialRequest {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), None, None),
    );
    worth_execution::SerialRequest::from_policy(&policy, CancellationToken::new(), None)
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

pub fn assert_source_runtime(
    runtime: &WorthQueryProgramApplicationRuntime<TemporalHostSchema, TemporalExampleProgram>,
) {
    let scope = product_workflow_support::adapters::request_scope();
    let principal = product_workflow_support::adapters::block_on(
        admit_identity_adapter(runtime.installed_schema()).authenticate((), &scope),
    )
    .expect("the example identity must authenticate");
    let row = runtime
        .request(&principal, &scope)
        .on_branch(runtime.current_world())
        .query(TemporalIntentRead {
            identity: "intent-1".to_owned(),
        })
        .execute()
        .expect("the application entry must execute the exact product read")
        .rows()[0]
        .clone();
    assert_eq!(row.input, "payload");
}
