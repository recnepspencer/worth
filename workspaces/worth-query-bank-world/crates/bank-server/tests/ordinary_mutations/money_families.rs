//! All three retained money declarations and exact original replay use owning native custody.
use super::*;
pub(super) fn exercise(fixture: &fixture::OrdinaryReadFixture) {
    let teller = fixture.authenticate(TELLER);
    let owner = fixture.authenticate(OWNER);
    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&teller, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::deposit(Deposit {
                institution: fixture.institution,
                account: fixture.recipient_account,
                amount: Money::from_minor(500).unwrap(),
            }))
            .as_principal(&teller)
            .controls(BankMutationControls::new(scope.clone(), key("deposit")))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("money construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
        else {
            panic!("fresh money completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            ..
        } = done.evidence()
        else {
            panic!("fresh money evidence")
        };
        assert!(receipt.emitted_effect_count() > 0);
        let work = receipt.mutation_work().unwrap();
        assert!(work.decision_fact_count() > 0);
        assert!(work.proposed_fact_count() > 0);
        assert!(work.relational_invariant_execution_count() > 0);
        assert!(work.invariant_work_units() > 0);
        assert!(done.close(&request).is_ok());
    }
    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&teller, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::withdraw(Withdraw {
                institution: fixture.institution,
                account: fixture.recipient_account,
                amount: Money::from_minor(100).unwrap(),
            }))
            .as_principal(&teller)
            .controls(BankMutationControls::new(scope.clone(), key("withdraw")))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("money construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
        else {
            panic!("fresh money completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            ..
        } = done.evidence()
        else {
            panic!("fresh money evidence")
        };
        assert!(receipt.emitted_effect_count() > 0);
        let work = receipt.mutation_work().unwrap();
        assert!(work.decision_fact_count() > 0);
        assert!(work.proposed_fact_count() > 0);
        assert!(work.relational_invariant_execution_count() > 0);
        assert!(work.invariant_work_units() > 0);
        assert!(done.close(&request).is_ok());
    }
    {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&owner, &scope);
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::send_money(SendMoney {
                from: fixture.personal_account,
                recipient: principal_id(RECIPIENT),
                amount: Money::from_minor(250).unwrap(),
            }))
            .as_principal(&owner)
            .controls(BankMutationControls::new(scope.clone(), key("send")))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("money construction: {cause:?}"));
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
        else {
            panic!("fresh money completion")
        };
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt,
            ..
        } = done.evidence()
        else {
            panic!("fresh money evidence")
        };
        assert!(receipt.emitted_effect_count() > 0);
        let work = receipt.mutation_work().unwrap();
        assert!(work.decision_fact_count() > 0);
        assert!(work.proposed_fact_count() > 0);
        assert!(work.relational_invariant_execution_count() > 0);
        assert!(work.invariant_work_units() > 0);
        assert!(done.close(&request).is_ok());
    }
    let scope = request_scope();
    let request = fixture.world.runtime.request(&owner, &scope);
    let retry = fixture
        .world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            from: fixture.personal_account,
            recipient: principal_id(RECIPIENT),
            amount: Money::from_minor(250).unwrap(),
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(scope.clone(), key("send")))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("retry construction: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Progress(retry) =
        retry.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("retained replay pending")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(retry) =
        retry.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("retained replay completion")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Recovered {
        receipt,
        ..
    } = retry.evidence()
    else {
        panic!("recovered native evidence")
    };
    assert_eq!(
        receipt.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Recovered
    );
    assert_eq!(receipt.terminal().attempt_resources_released(), None);
    assert!(retry.close(&request).is_ok());
}
