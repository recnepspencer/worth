//! Bootstrap's seams consume real execution refusals; production has no lease injector.

use std::num::NonZeroUsize;
use std::sync::OnceLock;
use worth_execution::CancellationSource;
use worth_execution::CancellationToken;
use worth_execution::ExecutionAuthority;
use worth_execution::ExecutionAuthorityConfig;
use worth_execution::LeaseRequest;
use worth_foundational::facade::aspects;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectIdentity;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::LocatorAuthority;
use worth_foundational::facade::ScalarAspectType;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::bootstrap_publication::identity_indexes_built;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as QueryKind;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::indexes::DerivedIndexBuildOutcome;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::indexes::DerivedIndexDefinition;
use worth_relational::facade::indexes::DerivedIndexExecutionDenialKind;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::indexes::DerivedIndexKind;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::AspectBinding;
use worth_relational::facade::schema::DeclaredAspectContractBinding;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::BulkEntityCreateIntent;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use worth_relational::facade::transactions::WorkerIntentBatch;
pub fn isolated(test_name: &str, run: impl FnOnce()) {
    let test_name = test_name.split_once("::").unwrap().1;
    const PROBE: &str = "WORTH_BOOTSTRAP_INDEX_EXECUTION_PROBE";
    if std::env::var(PROBE).as_deref() == Ok(test_name) {
        run();
        return;
    }
    let output = crate::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name, "--nocapture"])
            .env(PROBE, test_name),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}
pub fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::MIN,
            charged_memory_bytes: None,
        })
        .unwrap()
    })
}
pub fn request() -> LeaseRequest {
    request_with_memory(None)
}
pub fn request_with_memory(charged_memory_bytes: Option<u64>) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, charged_memory_bytes, Some(1_000_000)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    }
}
pub fn refused_index_build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: LeaseRequest,
) -> DerivedIndexBuildOutcome {
    let (runtime, build_request) = index_fixture(execution);
    let baseline = runtime
        .index_authority()
        .build_for_commit(execution, build_request.clone());
    assert!(baseline.execution_denial.is_none());

    assert_eq!(baseline.generations.len(), 1);
    assert!(identity_indexes_built(baseline, 1).is_ok());
    let lease = authority().request_lease(request).unwrap();
    runtime.index_authority().build_for_commit(
        &worth_execution::ExecutionRequest::leased(&lease),
        build_request,
    )
}
use crate::domain_computation::primary_graph::bootstrap_publication::execution_refusals::unleased_fixture::index_fixture;

#[path = "unleased_fixture/mod.rs"]
pub mod unleased_fixture;
