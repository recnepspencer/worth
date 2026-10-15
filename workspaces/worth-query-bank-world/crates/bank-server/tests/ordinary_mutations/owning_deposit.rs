//! Bank retained deposit custody crosses real partial publication and fresh admission.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRecoveryRequestDenial, WorthQueryOperationCloseDenial,
    WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance as Advance, WorthQuerySourceOnlyDenial,
    WorthQuerySourceOnlyPosture,
};

#[test]
fn owning_retained_deposit_recovers_once_with_fresh_authority_and_cleanup() {
    let fixture = ordinary_read_world("owning-retained-deposit", 0);
    let reader = fixture.authenticate(OWNER);
    let before = account_activity(&fixture, &reader, fixture.personal_account).len();
    let teller = fixture.authenticate(TELLER);
    let input = Deposit {
        institution: fixture.institution,
        account: fixture.personal_account,
        amount: Money::from_minor(17).unwrap(),
    };
    let scope = request_scope();
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::deposit(input))
        .as_principal(&teller)
        .controls(BankMutationControls::new(
            scope.clone(),
            key("owning-deposit"),
        ))
        .retain_operation()
        .unwrap_or_else(|denial| panic!("declared deposit: {denial:?}"));
    fixture.world.runtime.fail_next_durable_append_for_test();
    let request = fixture.world.runtime.request(&teller, &scope);
    let history = NonZeroUsize::new(128).unwrap();
    let operation = match operation.advance(&request, history) {
        Advance::Progress(operation) => operation,
        _ => panic!("actual append partial retains Bank operation"),
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    drop(request);
    drop(teller);
    drop(scope);
    // The real native owner crosses the transport worker's suspended receive frame.
    let worker_runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let operation = worker_runtime.block_on(async move {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<()>(1);
        let worker = tokio::spawn(async move {
            receiver.recv().await.expect("fresh transport command");
            operation
        });
        tokio::task::yield_now().await;
        sender.send(()).await.unwrap();
        worker.await.unwrap()
    });
    let stranger = fixture.authenticate(STRANGER);
    let refused_scope = request_scope();
    let refused = fixture.world.runtime.request(&stranger, &refused_scope);
    let operation = match operation.advance(&refused, history) {
        Advance::Refused {
            operation,
            denial:
                WorthQuerySourceOnlyDenial::Recovery(
                    WorthQueryApplicationRecoveryRequestDenial::Request(
                        WorthQueryApplicationRequestMutationDenial::Authorization(denial),
                    ),
                ),
        } => {
            assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied));
            operation
        }
        _ => panic!("fresh stranger must fail native deposit authorization"),
    };
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::ProductUnpublished
    );
    assert_eq!(
        account_activity(&fixture, &reader, fixture.personal_account).len(),
        before
    );
    drop(refused);
    drop(stranger);
    drop(refused_scope);
    let teller = fixture.authenticate(TELLER);
    let scope = request_scope();
    let request = fixture.world.runtime.request(&teller, &scope);
    let operation = match operation.advance(&request, history) {
        Advance::Progress(operation) => operation,
        _ => panic!("fresh authorized native recovery"),
    };
    let operation = match operation.advance(&request, history) {
        Advance::Progress(operation) => operation,
        _ => panic!("actual keyed evidence awaits retained commit delivery"),
    };
    let completed = match operation.advance(&request, history) {
        Advance::Completed(completed) => completed,
        _ => panic!("explicit history bound delivers original World commit"),
    };
    let WorthQuerySourceCompletion::Recovered {
        receipt,
        retained: Some(retained),
        publication: Some(_),
        ..
    } = completed.evidence()
    else {
        panic!("recovery supplies native evidence, never an invented MoneyMovementResult")
    };
    assert_eq!(
        retained.selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(receipt.precondition_comparison().expected_fact_count(), 0);
    let original = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&reader)
        .retained(retained, read_controls())
        .unwrap();
    assert_eq!(original.rows()[0].entries().len(), before + 1);
    assert_eq!(
        original.rows()[0]
            .entries()
            .iter()
            .filter(|entry| entry.amount().minor_units() == 17)
            .count(),
        1
    );
    assert_eq!(
        account_activity(&fixture, &reader, fixture.personal_account).len(),
        before + 1
    );
    let later = fixture
        .world
        .runtime
        .mutate(mutations::deposit(Deposit {
            institution: fixture.institution,
            account: fixture.personal_account,
            amount: Money::from_minor(19).unwrap(),
        }))
        .as_principal(&teller)
        .controls(BankMutationControls::new(
            scope.clone(),
            key("owning-deposit-later"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("later deposit construction: {cause:?}"));
    let Advance::Completed(later) =
        later.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("later deposit")
    };
    assert!(later.close(&request).is_ok());
    let original = fixture
        .world
        .runtime
        .account_activity(fixture.personal_account)
        .as_principal(&reader)
        .retained(retained, read_controls())
        .unwrap();
    assert_eq!(original.rows()[0].entries().len(), before + 1);
    assert_eq!(
        account_activity(&fixture, &reader, fixture.personal_account).len(),
        before + 2
    );
    let cancelled = WorthQueryCancellationSource::new();
    cancelled.cancel();
    let stopped_scope = WorthQueryRequestScope::new(None, cancelled.token());
    let stopped = fixture.world.runtime.request(&teller, &stopped_scope);
    let (completed, denial) = completed
        .close(&stopped)
        .err()
        .expect("native cleanup admission refuses cancelled request");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    drop(request);
    drop(teller);
    drop(scope);
    let closer = fixture.authenticate(TELLER);
    let close_scope = request_scope();
    let close = fixture.world.runtime.request(&closer, &close_scope);
    let _closed = completed
        .close(&close)
        .unwrap_or_else(|(_, denial)| panic!("fresh lawful native close: {denial:?}"));
}

#[test]
fn money_actions_install_exact_retained_meaning_and_deployment_authority() {
    use bank_domain::schema::{BankProgramDeploymentBinding, BankSchema};
    use worth_query_host::facade::declaration::application_program::*;
    fn verify<P: ApplicationProgramDefinition<BankSchema>>() {
        let program = ApplicationProgramAuthoring::<BankSchema, P>::begin()
            .validated_program()
            .unwrap();
        let expected = [
            <bank_server::OpeningFundingAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::ApplyOpeningFundingMutationBinding,
            >>::declaration(),
            <bank_server::DepositAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::DepositMutationBinding,
            >>::declaration(),
            <bank_server::WithdrawalAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::WithdrawMutationBinding,
            >>::declaration(),
            <bank_server::SendMoneyAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::SendMoneyMutationBinding,
            >>::declaration(),
            <bank_server::ReverseJournalAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::ReverseJournalMutationBinding,
            >>::declaration(),
        ];
        let mut identities = std::collections::BTreeSet::new();
        for action in expected {
            let installed = program
                .actions()
                .iter()
                .filter(|installed| installed.binding() == action.binding())
                .collect::<Vec<_>>();
            assert_eq!(installed.len(), 1);
            assert_eq!(
                installed[0].composition_instance(),
                action.composition_instance()
            );
            assert_eq!(installed[0].feature(), action.feature());
            assert!(!installed[0].required_output_source());
            let delivery = installed[0]
                .delivery()
                .expect("money action installs explicit delivery");
            assert!(matches!(
                delivery,
                ApplicationActionDeliveryDeclaration::RetainedCommit { .. }
            ));
            assert_eq!(
                delivery.canonical_meaning(),
                action.delivery().unwrap().canonical_meaning()
            );
            assert!(identities.insert(delivery.identity()));
        }
        assert_eq!(
            program
                .adoption_authorization()
                .unwrap()
                .canonical_meaning(),
            ApplicationProgramAdoptionAuthorizationDeclaration::from_binding::<
                BankSchema,
                BankProgramDeploymentBinding,
            >()
            .canonical_meaning()
        );
    }
    verify::<bank_server::BankApplication>();
    verify::<bank_server::BankApplicationP1>();
    verify::<bank_server::BankApplicationP2>();
}
