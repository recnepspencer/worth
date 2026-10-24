use crate::facade::config::RelationalRuntimeProfile;
use crate::facade::config::{
    MvccConfig, RetentionBackend, SnapshotReleasePolicy, VisibilityCachePolicy,
};
use crate::facade::runtime::RelationalRuntime;
use crate::tests::harness::fixtures::runtime::RuntimeHarnessMode;
use crate::tests::support::test_schema_registry;

pub(crate) fn build_property_runtime(mode: RuntimeHarnessMode) -> RelationalRuntime {
    match mode {
        RuntimeHarnessMode::InMemory(profile) => {
            crate::facade::runtime::RelationalRuntimeApi::builder()
                .profile(profile)
                .schema_registry(test_schema_registry())
                .mvcc(property_mvcc_config())
                .visibility_cache_policy(VisibilityCachePolicy {
                    enabled: true,
                    protect_branch_heads: false,
                    protect_replay_retained: false,
                    protect_active_snapshots: true,
                    recent_version_window: 2,
                })
                .build()
        }
        RuntimeHarnessMode::Persisted => crate::facade::runtime::RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(test_schema_registry())
            .mvcc(property_mvcc_config())
            .build_durable(crate::tests::support::TestDurableMedium::new().attachment()),
    }
}

fn property_mvcc_config() -> MvccConfig {
    MvccConfig {
        track_visibility_metadata: true,
        snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
        auto_reclaim_deleted_records: true,
        reclaim_batch_size: 8,
        retention_backend: RetentionBackend::EpochChunkRetention,
    }
}
