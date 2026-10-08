use super::*;
use crate::facade::diagnostics::RelationalDiagnosticsProfile;
use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionResourceLease,
    LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

static TEST_STORE_COUNTER: AtomicU64 = AtomicU64::new(0);
static TEST_EXECUTION_AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();

pub(crate) fn test_execution_authority() -> &'static ExecutionAuthority {
    TEST_EXECUTION_AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).expect("positive test worker count"),
            charged_memory_bytes: Some(512 * 1024 * 1024),
        })
        .expect("one execution authority for relational lib tests")
    })
}

pub(crate) fn test_execution_lease() -> ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(4).expect("positive test worker count"),
                    32 * 1024 * 1024,
                    1_000_000,
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .expect("test execution lease admitted")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PerfDiagnosticsPolicy {
    GeometryOperationalHotPath,
    GeometryRichCertification,
    ChipOperationalHotPath,
    ChipRichCertification,
}

pub(crate) fn apply_perf_diagnostics_policy(
    runtime: &mut RelationalRuntime,
    policy: PerfDiagnosticsPolicy,
) {
    let profile = match policy {
        PerfDiagnosticsPolicy::GeometryOperationalHotPath => {
            RelationalDiagnosticsProfile::geometry_operational_hot_path()
        }
        PerfDiagnosticsPolicy::GeometryRichCertification => {
            RelationalDiagnosticsProfile::geometry_rich_certification()
        }
        PerfDiagnosticsPolicy::ChipOperationalHotPath => {
            RelationalDiagnosticsProfile::chip_operational_hot_path()
        }
        PerfDiagnosticsPolicy::ChipRichCertification => {
            RelationalDiagnosticsProfile::chip_rich_certification()
        }
    };
    runtime.configure_diagnostics_for_test(|configured| *configured = profile);
}

pub(crate) fn runtime_with_test_schema() -> RelationalRuntime {
    runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore)
}

pub(crate) fn snapshot_for_owner_identity(
    runtime: &RelationalRuntime,
    identity: &crate::branch::RelationalBranchIdentity,
) -> crate::snapshots::data::SnapshotHandle {
    let (_, basis) = runtime
        .observe_branch(identity)
        .expect("test branch basis is owner-admitted");
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .expect("owner-admitted observation opens its exact snapshot")
}

pub(crate) fn snapshot_for_owner_branch(
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) -> crate::snapshots::data::SnapshotHandle {
    let identity = runtime
        .branch_identity(branch_id)
        .expect("test branch identity is owner-issued");
    snapshot_for_owner_identity(runtime, &identity)
}

pub(crate) fn runtime_with_declared_aspect_schema_profile(
    profile: RelationalRuntimeProfile,
    cascade_delete_policy: CascadeDeletePolicy,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(profile)
        .schema_registry(declared_aspect_schema_registry(cascade_delete_policy))
        .build()
}

pub(crate) fn runtime_with_test_schema_profile(
    profile: RelationalRuntimeProfile,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(profile)
        .schema_registry(test_schema_registry())
        .build()
}

pub(crate) fn runtime_with_test_schema_profile_and_chunks(
    profile: RelationalRuntimeProfile,
    entity_chunk_size: usize,
    relation_chunk_size: usize,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(profile)
        .schema_registry(test_schema_registry())
        .storage_layout(StorageLayoutConfig {
            entity_chunk_size,
            relation_chunk_size,
            scan_packet_size: 256,
        })
        .build()
}

pub(crate) fn persisted_runtime_with_test_schema() -> RelationalRuntime {
    persisted_runtime_with_test_schema_profile(RelationalRuntimeProfile::CertificationCore)
}

pub(crate) fn persisted_runtime_with_test_schema_profile(
    profile: RelationalRuntimeProfile,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(profile)
        .schema_registry(test_schema_registry())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("worth-relational-persisted"),
            segment_commit_capacity: 2,
        })
        .build()
}

pub(crate) fn persisted_runtime_with_declared_aspect_schema(
    cascade_delete_policy: CascadeDeletePolicy,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(declared_aspect_schema_registry(cascade_delete_policy))
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("worth-relational-persisted-aspects"),
            segment_commit_capacity: 2,
        })
        .build()
}

pub(crate) fn unique_test_store_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEST_STORE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("{prefix}-{nanos}-{counter}"));
    let _ = fs::remove_dir_all(&path);
    path
}

pub(crate) fn runtime_with_test_schema_and_chunks(
    entity_chunk_size: usize,
    relation_chunk_size: usize,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(test_schema_registry())
        .storage_layout(StorageLayoutConfig {
            entity_chunk_size,
            relation_chunk_size,
            scan_packet_size: 64,
        })
        .build()
}

pub(crate) fn runtime_with_test_schema_and_invariants(
    invariant_catalog: InvariantCatalog,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .invariant_catalog(invariant_catalog)
        .build()
}

pub(crate) fn runtime_with_declared_aspect_schema_and_invariants(
    invariant_catalog: InvariantCatalog,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .invariant_catalog(invariant_catalog)
        .build()
}
