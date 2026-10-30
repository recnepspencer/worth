#[allow(unused_imports)]
use crate::sealed_runtime_lifecycle_journey::*;

#[test]
fn sealed_runtime_lifecycle_journey_establishes_the_admission_foundation() {
    let counters_before = worth_store::physical_runtime::PhysicalStore::diagnostics();
    prove_invalid_admission_is_allocation_free();
    prove_declared_root_closes_without_retaining_authority();
    prove_existing_root_is_unchanged();
    assert_process_counter_delta(
        counters_before,
        worth_store::physical_runtime::PhysicalStore::diagnostics(),
    );
}
