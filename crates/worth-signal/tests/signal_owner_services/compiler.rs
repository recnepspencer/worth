//! Grouped public-facade compiler evidence for owner-service boundaries.

#[test]
fn owner_services_public_facade_and_negative_fences_are_current() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/signal_owner_services/ui/public_port_matrix.rs");
    tests.pass("tests/signal_owner_services/ui/conditional_claimant_issuance.rs");
    tests.compile_fail(
        "tests/signal_owner_services/ui/component_bundle_has_no_conditional_issuer.rs",
    );
    tests.compile_fail("tests/signal_owner_services/ui/conditional_issuance_is_owner_private.rs");
    #[cfg(feature = "test-operation-control")]
    tests.pass("tests/signal_owner_services/ui/operation_control_matrix.rs");
    #[cfg(feature = "test-operation-control")]
    tests
        .compile_fail("tests/signal_owner_services/ui/operation_control_constructor_is_private.rs");
    tests.compile_fail("tests/signal_owner_services/ui/local_only_owner_issuance.rs");
    tests.compile_fail("tests/signal_owner_services/ui/descriptor_is_not_basis_authority.rs");
    tests.compile_fail("tests/signal_owner_services/ui/raw_branch_id_is_not_basis_authority.rs");
    tests.compile_fail("tests/signal_owner_services/ui/basis_port_cannot_mutate.rs");
    tests.compile_fail("tests/signal_owner_services/ui/retirement_requires_linear_plan.rs");
    tests.compile_fail("tests/signal_owner_services/ui/consumed_outcome_cannot_reuse.rs");
    tests.compile_fail("tests/signal_owner_services/ui/generic_marker_cannot_issue_basis.rs");
    tests.compile_fail("tests/signal_owner_services/ui/forged_basis_is_not_public.rs");
    tests.compile_fail("tests/signal_owner_services/ui/private_cell_is_not_facade.rs");
    #[cfg(not(feature = "test-operation-control"))]
    tests.compile_fail("tests/signal_owner_services/ui/default_build_has_no_operation_control.rs");
    tests.compile_fail("tests/signal_owner_services/ui/unavailable_is_not_constructible.rs");
    tests.compile_fail("tests/signal_owner_services/ui/ports_are_not_constructible.rs");
}
