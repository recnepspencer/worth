#[test]
fn public_authority_boundaries() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/valid_map.rs");
    cases.compile_fail("tests/ui/forge_lease.rs");
    cases.compile_fail("tests/ui/forge_map.rs");
}

#[test]
fn dropped_authority_cannot_be_reconstructed() {
    use std::num::NonZeroUsize;
    use worth_execution::{ConstructionDenial, ExecutionAuthority, ExecutionAuthorityConfig};

    let config = ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: 1,
    };
    drop(ExecutionAuthority::try_construct(config).unwrap());
    assert_eq!(
        ExecutionAuthority::try_construct(config).unwrap_err(),
        ConstructionDenial::AlreadyConstructed,
    );
}
