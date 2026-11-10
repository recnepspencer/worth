#![cfg_attr(test, allow(clippy::disallowed_methods))]
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
        let path = format!("{}/tests/worth_store_court_store/sources/worth_store/physical_adapter_authority/{denied}", env!("CARGO_MANIFEST_DIR"));
        if denied == "successor_allocation_forgery_is_rejected.rs" {
            worth_testkit_compiler::certification_semantic_denial(
                &path,
                "E0451",
                &["field `allocation` of struct `RecoveryPhysicalAllocation` is private"],
            )
            .expect("the actual recovery allocation field stays private");
            worth_testkit_compiler::certification_uncoded_semantic_denial(
                &path,
                "is private",
                &["type `physical_runtime::record_serving::residency::scoped_allocation::grant::StoreScopedAllocation<'_, physical_runtime::record_serving::residency::scoped_allocation::scope::RecoveryScope>` is private"],
            )
            .expect("the actual scoped recovery allocation type stays private");
        } else {
            worth_testkit_compiler::denied(&path).expect("prebuilt compiler denial");
        }
    }
}

#[test]
fn certification_authority_is_absent_in_the_production_environment() {
    worth_testkit_compiler::production_denial(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store/sources/worth_store/physical_adapter_authority/certification_authority_is_absent.rs")).expect("production compiler denial");
}
