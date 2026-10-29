#[test]
fn worker_boundary_internal_type_surfaces_are_not_product_api() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(
        "tests/compile_fail/type_surface/product_code_cannot_import_placement_declaration_candidate.rs",
    );
    cases.compile_fail(
        "tests/compile_fail/type_surface/product_code_cannot_import_worker_boundary_artifact_lock.rs",
    );
    cases.compile_fail(
        "tests/compile_fail/worth_proof_progression/unresolved_recipe_is_not_resolved.rs",
    );
    cases.compile_fail(
        "tests/compile_fail/worth_proof_progression/resolved_recipe_is_not_lowered.rs",
    );
    cases.compile_fail(
        "tests/compile_fail/worth_proof_progression/lowered_recipe_is_not_execution_ready.rs",
    );
    cases.compile_fail(
        "tests/compile_fail/worth_proof_progression/unresolved_recipe_is_not_readmitted.rs",
    );
}
