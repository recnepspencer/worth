//! Original account input survives real partial publication and fresh-authority recovery.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRecoveryRequestDenial, WorthQueryOperationCloseDenial,
    WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
    WorthQuerySourceOnlyDenial, WorthQuerySourceOnlyPosture,
};
#[test]
fn owning_personal_account_partial_refuses_recovers_reads_and_closes_once() {
    let fixture = ordinary_read_world("owning-personal-partial", 0);
    let input = CreatePersonalAccount {
        institution: fixture.institution,
        owner: principal_id(STRANGER),
        display_name: AccountName::new("Recovered original account").unwrap(),
    };
    let operation = {
        let teller = fixture.authenticate(TELLER);
        let scope = request_scope();
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::create_personal_account(input.clone()))
            .as_principal(&teller)
            .controls(BankMutationControls::new(
                scope.clone(),
                key("owning-personal-partial"),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("declared account action: {cause:?}"));
        fixture.world.runtime.fail_next_durable_append_for_test();
        let request = fixture.world.runtime.request(&teller, &scope);
        let Advance::Progress(operation) = operation.advance(&request) else {
            panic!("authentic partial publication")
        };
        assert_eq!(
            operation.posture(),
            WorthQuerySourceOnlyPosture::ProductUnpublished
        );
        operation
    };
    let stranger = fixture.authenticate(STRANGER);
    let operation = {
        let scope = request_scope();
        let request = fixture.world.runtime.request(&stranger, &scope);
        let operation = match operation.advance(&request) {
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
            _ => panic!("account owner cannot substitute fresh teller permission"),
        };
        assert_eq!(
            operation.posture(),
            WorthQuerySourceOnlyPosture::ProductUnpublished
        );
        assert!(request
            .query(queries::accounts())
            .execute()
            .unwrap()
            .rows()
            .is_empty());
        operation
    };
    let read_scope = request_scope();
    let read = fixture.world.runtime.request(&stranger, &read_scope);
    let completed = {
        let teller = fixture.authenticate(TELLER);
        let scope = request_scope();
        let request = fixture.world.runtime.request(&teller, &scope);
        let Advance::Progress(operation) = operation.advance(&request) else {
            panic!("fresh native recovery")
        };
        let Advance::Completed(completed) = operation.advance(&request) else {
            panic!("resolve genuine recovered commit")
        };
        let Completion::Recovered {
            receipt,
            publication: Some(_),
            ..
        } = completed.evidence()
        else {
            panic!("receipt evidence, no manufactured handler result")
        };
        assert_eq!(receipt.precondition_comparison().expected_fact_count(), 0);
        let accounts = read.query(queries::accounts()).execute().unwrap();
        let [account] = accounts.rows() else {
            panic!("one independently visible account")
        };
        let detail = read
            .query(queries::account_detail(account.id()))
            .execute()
            .unwrap();
        assert_eq!(
            detail.rows()[0].summary().display_name(),
            &input.display_name
        );
        assert_eq!(detail.rows()[0].personal_owner(), Some(input.owner));
        assert_eq!(detail.rows()[0].institution(), input.institution);
        let cancelled = WorthQueryCancellationSource::new();
        cancelled.cancel();
        let stopped_scope = WorthQueryRequestScope::new(None, cancelled.token());
        let stopped = fixture.world.runtime.request(&teller, &stopped_scope);
        let (completed, denial) = completed
            .close(&stopped)
            .err()
            .expect("close retains native cleanup on refusal");
        assert!(matches!(
            denial,
            WorthQueryOperationCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            ))
        ));
        completed
    };
    let teller = fixture.authenticate(TELLER);
    let scope = request_scope();
    let request = fixture.world.runtime.request(&teller, &scope);
    completed
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("fresh lawful cleanup: {cause:?}"));
    assert_eq!(
        read.query(queries::accounts())
            .execute()
            .unwrap()
            .rows()
            .len(),
        1
    );
}
#[test]
fn plain_actions_install_exact_completion_meaning_and_scoped_deployment_authority() {
    use bank_domain::schema::*;
    use worth_query_host::facade::declaration::application_program::*;
    fn verify<P: ApplicationProgramDefinition<BankSchema>>() {
        let program = ApplicationProgramAuthoring::<BankSchema, P>::begin()
            .validated_program()
            .unwrap();
        let actions = [
            <bank_server::PersonalAccountCreationAction as ApplicationAction<
                BankSchema,
                CreatePersonalAccountMutationBinding,
            >>::declaration(),
            <bank_server::BusinessAccountCreationAction as ApplicationAction<
                BankSchema,
                CreateBusinessAccountMutationBinding,
            >>::declaration(),
            <bank_server::AccountAccessGrantAction as ApplicationAction<
                BankSchema,
                GrantAccountAccessMutationBinding,
            >>::declaration(),
            <bank_server::AccountAccessRevocationAction as ApplicationAction<
                BankSchema,
                RevokeAccountAccessMutationBinding,
            >>::declaration(),
            <bank_server::PaymentInitiationAction as ApplicationAction<
                BankSchema,
                InitiateBusinessPaymentMutationBinding,
            >>::declaration(),
            <bank_server::PaymentRejectionAction as ApplicationAction<
                BankSchema,
                RejectPaymentMutationBinding,
            >>::declaration(),
        ];
        let mut identities = std::collections::BTreeSet::new();
        for action in actions {
            let installed = program
                .actions()
                .iter()
                .filter(|item| item.binding() == action.binding())
                .collect::<Vec<_>>();
            assert_eq!(installed.len(), 1);
            assert_eq!(
                installed[0].composition_instance(),
                action.composition_instance()
            );
            assert_eq!(installed[0].feature(), action.feature());
            assert!(!installed[0].required_output_source());
            let delivery = installed[0].delivery().unwrap();
            assert!(matches!(
                delivery,
                ApplicationActionDeliveryDeclaration::SourceCompletion { .. }
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
