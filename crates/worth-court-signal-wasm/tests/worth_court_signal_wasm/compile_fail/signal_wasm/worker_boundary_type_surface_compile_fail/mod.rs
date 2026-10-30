#[test]
fn worker_boundary_internal_type_surfaces_are_not_product_api() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_wasm/sources/worth_signal_wasm/compile_fail/type_surface/product_code_cannot_import_placement_declaration_candidate.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_wasm/sources/worth_signal_wasm/compile_fail/type_surface/product_code_cannot_import_worker_boundary_artifact_lock.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_wasm/sources/worth_signal_wasm/compile_fail/worth_proof_progression/unresolved_recipe_is_not_resolved.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_wasm/sources/worth_signal_wasm/compile_fail/worth_proof_progression/resolved_recipe_is_not_lowered.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_wasm/sources/worth_signal_wasm/compile_fail/worth_proof_progression/lowered_recipe_is_not_execution_ready.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_wasm/sources/worth_signal_wasm/compile_fail/worth_proof_progression/unresolved_recipe_is_not_readmitted.rs")).expect("prebuilt compiler denial");
}
