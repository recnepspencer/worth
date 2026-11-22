//! A source operation stopped by a full bounded history is a no-effect
//! refusal: its terminal closes and releases the Home.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQuerySourceOnlyAdvance,
    application_installation::WorthQueryBoundedApplicationLimits,
    runtime::RuntimeWorldHistoryBudgetInstallation,
};

#[test]
fn history_capacity_refusal_closes_its_terminal_source_operation() {
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let single_commit = RuntimeWorldHistoryBudgetInstallation {
        retained_composite_commits: 1,
        ..WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS.history
    };
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .with_history(single_commit)
        .install()
        .unwrap();
    let home = open_home(&execution, limits)
        .unwrap_or_else(|refused| panic!("the seeded Home fits one commit: {refused:?}"));
    let scope = scope(&clock);
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&scope)
        .unwrap();
    {
        let request = home.runtime().request(&principal, &scope);
        let operation = request
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 9,
            })
            .without_source()
            .idempotency_owned(0x4849_5354_u64)
            .unwrap_or_else(|(_, _, cause)| panic!("describe the archive-like edit: {cause:?}"))
            .retain_durable_operation::<fixture::HomeProgram, fixture::BalanceAction>(
                &request,
                home.runtime(),
            )
            .unwrap_or_else(|(_, cause)| panic!("retain the durable original: {cause:?}"));
        let terminal = match operation.advance(&request) {
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => terminal,
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => panic!("refused: {denial:?}"),
            WorthQuerySourceOnlyAdvance::Completed(_)
            | WorthQuerySourceOnlyAdvance::Progress(_) => {
                panic!("a full history cannot publish the edit")
            }
        };
        let outcome = format!("{:?}", terminal.native_outcome());
        assert!(
            outcome.contains("ProductPreparationRefused") && outcome.contains("CapacityExhausted"),
            "the World refuses before preparation: {outcome}"
        );
        terminal.close(&request).unwrap_or_else(|(_, cause)| {
            panic!("a no-effect preparation refusal closes: {cause:?}")
        });
    }
    assert_eq!(read_balance(&home, &principal, &scope), Ok(vec![7]));
    drop(principal);
    close_home(home, &execution);
}
