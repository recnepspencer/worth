#[test]
fn physical_adapters_receive_observation_without_pool_control() {
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
        worth_testkit_compiler::denied(format!("{}/tests/worth_store_court_store_entry/sources/worth_store/physical_adapter_authority/{denied}", env!("CARGO_MANIFEST_DIR"))).expect("prebuilt compiler denial");
    }
}

#[test]
fn certification_authority_is_absent_in_the_production_environment() {
    worth_testkit_compiler::production_denial(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_entry/sources/worth_store/physical_adapter_authority/certification_authority_is_absent.rs")).expect("production compiler denial");
}
