//! Plain account actions install their exact completion meaning and deployment authority.
#[test]
fn plain_actions_install_exact_completion_meaning_and_scoped_deployment_authority() {
    use bank_domain::schema::*;
    use worth_query_host::facade::declaration::application_program::*;
    fn verify<P: ApplicationProgramDefinition<BankSchema>>() {
        let program = ApplicationProgramAuthoring::<BankSchema, P>::begin()
            .validated_program()
            .unwrap();
        let expected_selectors = [
            <bank_server::PersonalAccountCreationAction as ApplicationAction<
                BankSchema,
                CreatePersonalAccountMutationBinding,
            >>::selector(),
            <bank_server::BusinessAccountCreationAction as ApplicationAction<
                BankSchema,
                CreateBusinessAccountMutationBinding,
            >>::selector(),
            <bank_server::AccountAccessGrantAction as ApplicationAction<
                BankSchema,
                GrantAccountAccessMutationBinding,
            >>::selector(),
            <bank_server::AccountAccessRevocationAction as ApplicationAction<
                BankSchema,
                RevokeAccountAccessMutationBinding,
            >>::selector(),
            <bank_server::PaymentInitiationAction as ApplicationAction<
                BankSchema,
                InitiateBusinessPaymentMutationBinding,
            >>::selector(),
            <bank_server::PaymentRejectionAction as ApplicationAction<
                BankSchema,
                RejectPaymentMutationBinding,
            >>::selector(),
        ];
        let mut identities = std::collections::BTreeSet::new();
        for selector in expected_selectors {
            let installed = program
                .actions()
                .iter()
                .filter(|item| item.binding() == selector.binding())
                .collect::<Vec<_>>();
            assert_eq!(installed.len(), 1);
            assert_eq!(
                installed[0].composition_instance(),
                selector.composition_instance()
            );
            assert_eq!(installed[0].feature(), selector.feature());
            assert_eq!(
                installed[0].mutation_binding_type(),
                Some(selector.mutation_binding_type())
            );
            assert!(!installed[0].required_output_source());
            let delivery = installed[0].delivery().unwrap();
            assert!(matches!(
                delivery,
                ApplicationActionDeliveryDeclaration::SourceCompletion { .. }
            ));
            assert_eq!(delivery, selector.delivery());
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
