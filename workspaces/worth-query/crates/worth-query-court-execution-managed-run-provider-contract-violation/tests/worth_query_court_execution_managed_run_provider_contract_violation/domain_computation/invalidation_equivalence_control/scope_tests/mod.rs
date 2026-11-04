use worth_query_execution::facade::domain_computation::WorthQueryInvalidationEquivalenceForTest;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationEquivalenceForTest;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
pub fn resources() -> WorthQueryInvalidationResources {
    let resources = WorthQueryInvalidationResources::new();
    resources.attach_runtime_observations_for_test();
    resources
}

#[test]
fn constructing_under_a_factory_covers_the_receivers_workers_and_ends_on_drop() {
    let ordinary = resources();
    let guard = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let controlled = resources();
    assert!(!ordinary.invalidation_equivalence_is_armed_for_test());
    let worker = controlled.clone();
    assert!(
        std::thread::spawn(move || worker.invalidation_equivalence_is_armed_for_test())
            .join()
            .unwrap()
    );
    drop(guard);
    assert!(!controlled.invalidation_equivalence_is_armed_for_test());
}
#[test]
fn resources_installed_here_carry_the_arm_into_worker_construction() {
    let guard = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let resources = WorthQueryInvalidationResources::new();
    let worker = resources.clone();
    assert!(std::thread::spawn(move || {
        worker.attach_runtime_observations_for_test();
        worker.invalidation_equivalence_is_armed_for_test()
    })
    .join()
    .unwrap());
    drop(guard);
    assert!(!resources.invalidation_equivalence_is_armed_for_test());
}
#[test]
fn unwinding_an_explicit_arm_clears_only_its_receiver() {
    let controlled = resources();
    let other = resources();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard =
            WorthQueryInvalidationEquivalenceForTest::arm_for_invalidation_resources_for_test(
                &controlled,
            );
        assert!(controlled.invalidation_equivalence_is_armed_for_test());
        assert!(!other.invalidation_equivalence_is_armed_for_test());
        panic!("exercise receiver oracle cleanup");
    }))
    .is_err());
    assert!(!controlled.invalidation_equivalence_is_armed_for_test());
}
