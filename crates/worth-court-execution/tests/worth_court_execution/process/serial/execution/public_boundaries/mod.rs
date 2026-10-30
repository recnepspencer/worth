#[allow(unused_imports)]
pub(crate) use {};

#[test]
fn dropped_authority_cannot_be_reconstructed() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    use std::num::NonZeroUsize;
    use {
        worth_execution::ConstructionDenial, worth_execution::ExecutionAuthority,
        worth_execution::ExecutionAuthorityConfig,
    };

    let config = ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(1).unwrap(),
        charged_memory_bytes: Some(1),
    };
    drop(ExecutionAuthority::try_construct(config).unwrap());
    assert_eq!(
        ExecutionAuthority::try_construct(config).unwrap_err(),
        ConstructionDenial::AlreadyConstructed,
    );
}
