pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::read_ceiling_terminal::*;

#[test]
fn conditional_read_ceiling_answers_the_computation_terminal_by_value() {
    let (terminal, cause, charges) = observe(64_000_000);
    assert_eq!(
        terminal,
        graph::WorthQueryConditionalExecutionTerminal::Committed
    );
    assert_eq!(cause, None);
    let expected = (
        graph::WorthQueryConditionalExecutionTerminal::Failed,
        Some(
            graph::WorthQueryConditionalExecutionCause::ApplicationExecutionDenied {
                stage: graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
                cause: Ok(
                    graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: Resource::WorkExhausted,
                        partition_identity: None,
                        policy_ancestor: None,
                    },
                ),
            },
        ),
    );
    // Existing charged checkpoints select caller ceilings without fixing a
    // price or an ordinal. The expected stage, terminal and cause are fixed
    // independently: renaming the stop leaves no matching answer.
    let mut answered = false;
    for required in charges
        .into_iter()
        .filter(|n| *n > 0)
        .collect::<std::collections::BTreeSet<_>>()
    {
        let (terminal, cause, _) = observe(required - 1);
        if (terminal, cause) == expected {
            answered = true;
            break;
        }
    }
    assert!(
        answered,
        "the real DecisionReadSet ceiling answers {expected:?}"
    );
}
