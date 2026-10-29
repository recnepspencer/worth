#[allow(unused_imports)] use crate::physical_record_journeys::successor_scope_admission::*;
use {std::num::NonZeroU32, std::num::NonZeroU64};

use {worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPressureEvidence, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_physical_format::store_namespace::StableStoreIdentity};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};

macro_rules! assert_exact_scope_ceiling {
    ($serving:expr, $allocations:expr, $store:expr, $generation:expr, $admit:ident, $scope:expr) => {{
        let exact = $allocations
            .$admit(bytes(PAGE_BYTES * 2))
            .expect("the exact successor scope ceiling must be admitted");
        assert_eq!(exact.store_identity(), $store);
        assert_eq!(exact.runtime_identity(), $serving.runtime_identity());
        assert_eq!(exact.store_generation(), $generation);
        assert_eq!(exact.bytes(), PAGE_BYTES * 2);
        assert_eq!(
            $serving
                .residency_observation()
                .counters()
                .active_operation_bytes_for($scope),
            PAGE_BYTES * 2,
        );
        let pressure = $allocations
            .$admit(bytes(1))
            .expect_err("one byte past the live scope ceiling must be denied")
            .pressure()
            .expect("scope exhaustion must lower to Store pressure evidence");
        assert_pressure(
            pressure,
            ExpectedScopePressure {
                store: $store,
                dimension: worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope($scope),
                scope: $scope,
                requested: 1,
                admitted: PAGE_BYTES * 2,
                limit: PAGE_BYTES * 2,
            },
        );
        drop(exact);
        assert_eq!(
            $serving
                .residency_observation()
                .counters()
                .active_operation_bytes_for($scope),
            0,
        );
    }};
}

#[test]
fn successor_scopes_are_exact_isolated_global_and_released() {
    let root = tempfile::tempdir().unwrap();
    let serving = successor_scope_runtime(root.path());
    assert_exact_scope_ceilings(&serving);
    assert_scope_isolation(&serving);
    assert_global_envelope_and_release(&serving);
}
