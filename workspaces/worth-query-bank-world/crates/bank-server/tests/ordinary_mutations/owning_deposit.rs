//! Money actions install their exact retained meaning and deployment authority.

#[test]
fn money_actions_install_exact_retained_meaning_and_deployment_authority() {
    use bank_domain::schema::{BankProgramDeploymentBinding, BankSchema};
    use worth_query_host::facade::declaration::application_program::*;
    fn verify<P: ApplicationProgramDefinition<BankSchema>>() {
        let program = ApplicationProgramAuthoring::<BankSchema, P>::begin()
            .validated_program()
            .unwrap();
        let expected_selectors = [
            <bank_server::OpeningFundingAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::ApplyOpeningFundingMutationBinding,
            >>::selector(),
            <bank_server::DepositAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::DepositMutationBinding,
            >>::selector(),
            <bank_server::WithdrawalAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::WithdrawMutationBinding,
            >>::selector(),
            <bank_server::SendMoneyAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::SendMoneyMutationBinding,
            >>::selector(),
            <bank_server::ReverseJournalAction as ApplicationAction<
                BankSchema,
                bank_domain::schema::ReverseJournalMutationBinding,
            >>::selector(),
        ];
        let mut identities = std::collections::BTreeSet::new();
        for selector in expected_selectors {
            let installed = program
                .actions()
                .iter()
                .filter(|installed| installed.binding() == selector.binding())
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
            let delivery = installed[0]
                .delivery()
                .expect("money action installs explicit delivery");
            assert!(matches!(
                delivery,
                ApplicationActionDeliveryDeclaration::RetainedCommit { .. }
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
