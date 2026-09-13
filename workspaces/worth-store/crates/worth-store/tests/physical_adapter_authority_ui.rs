#[test]
fn physical_adapters_receive_observation_without_pool_control() {
    let cases = trybuild::TestCases::new();
    for supported in [
        "physical_integrity_supported.rs",
        "owner_disposition_observation_supported.rs",
        "physical_isolation_supported.rs",
        "blob_streaming_supported.rs",
        "operation_scopes_supported.rs",
        "successor_scoped_allocations_supported.rs",
    ] {
        cases.pass(format!("tests/physical_adapter_authority/{supported}"));
    }
    for denied in [
        "pool_construction_is_sealed.rs",
        "eviction_authority_is_sealed.rs",
        "dirty_mutation_authority_is_sealed.rs",
        "generation_forgery_is_rejected.rs",
        "semantic_residency_inference_is_rejected.rs",
        "owner_disposition_forgery_is_rejected.rs",
        "successor_scope_substitution_is_rejected.rs",
        "successor_allocation_forgery_is_rejected.rs",
        "successor_lower_grant_is_sealed.rs",
        "successor_allocation_cannot_escape_runtime.rs",
        "successor_runtime_cannot_close_with_live_allocation.rs",
    ] {
        cases.compile_fail(format!("tests/physical_adapter_authority/{denied}"));
    }
    #[cfg(not(feature = "certification-test-authority"))]
    cases.compile_fail("tests/physical_adapter_authority/certification_authority_is_absent.rs");
}
