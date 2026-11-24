#[cfg(test)]
mod tests {
    use super::super::aspect_version_read::read_versions;
    use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
    use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
    use crate::storage::substrate::AspectVersions;
    use crate::symbols::data::Symbol;
    use crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind;
    use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    fn versions(first: &str, second: &str) -> (RuntimeSymbolTable, AspectVersions) {
        let symbols = RuntimeSymbolTable::default();
        let mut versions = AspectVersions::default();
        symbols.normalize_client_keys(|interner| {
            [first, second]
                .into_iter()
                .enumerate()
                .map(|(revision, text)| {
                    let symbol = interner.intern(text).unwrap();
                    versions.insert(symbol, revision as u64 + 1);
                    (symbol, text.to_owned())
                })
                .collect()
        });
        (symbols, versions)
    }

    fn assert_native_budget_denials(first: &str, second: &str, sort: bool) {
        let (symbols, versions) = versions(first, second);
        for (memory, work, expected_work) in [(64 * 1024 * 1024, 1, true), (0, 100, false)] {
            let lease = worth_harness::facade::execution_fixture::test_execution_authority()
                .request_lease(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        ExecutionPosture::Serial,
                        DeterminismContract::CanonicalBitwise,
                        ExecutionBudget::new(
                            std::num::NonZeroUsize::new(1).unwrap(),
                            Some(memory),
                            Some(work),
                        ),
                    ),
                    deadline: None,
                    cancellation: CancellationToken::new(),
                })
                .unwrap();
            let stop = read_versions(&symbols, &ExecutionRequest::leased(&lease), &versions, sort)
                .unwrap_err();
            let kind = stop.execution_denial().unwrap().kind;
            if expected_work {
                assert_eq!(kind, CommitExecutionDenialKind::Cause(Cause::WorkExhausted));
            } else {
                assert!(matches!(
                    kind,
                    CommitExecutionDenialKind::Cause(Cause::PolicyMemoryExhausted {
                        admitted: 0,
                        ..
                    })
                ));
            }
        }
    }

    #[test]
    fn aspect_version_enumeration_spends_the_request_and_keeps_native_causes() {
        assert_native_budget_denials("lifecycle", "name", false);
    }

    #[test]
    fn relation_aspect_version_enumeration_spends_the_request_and_keeps_native_causes() {
        assert_native_budget_denials("lifecycle", "label", true);
    }

    #[test]
    fn unresolved_symbolic_aspect_versions_are_not_exposed_as_foundational_keys() {
        let host_request = worth_harness::facade::execution_fixture::serial_request();
        let execution = &ExecutionRequest::serial(&host_request);
        let symbols = RuntimeSymbolTable::default();
        let mut symbolic = AspectVersions::default();
        symbolic.insert(Symbol(41), 7);

        let observed = read_versions(&symbols, execution, &symbolic, false)
            .expect("aspect-version request remains live");

        assert!(observed.is_empty());
    }
}
