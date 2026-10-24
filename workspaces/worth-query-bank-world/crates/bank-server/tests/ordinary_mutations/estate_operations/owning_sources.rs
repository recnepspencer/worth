//! Genuine source custody and post-cleanup evidence, distinct from published effect recovery.
use super::freeze_account::fixture::exact_freeze_world;
use crate::support::request_scope;
use bank_domain::{
    proposals::BankIdempotencyKey,
    schema::{AccountStatus, FreezeEstateAccount},
};
use bank_server::{mutations, BankMutationControls};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
    },
};
#[test]
fn completed_source_refused_close_preserves_original_performed_change_witness() {
    let fixture = exact_freeze_world("owning-freeze-witness", AccountStatus::Open);
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let source = fixture
        .world
        .runtime
        .mutate(mutations::freeze_estate_account(
            FreezeEstateAccount::try_from(fixture.action(fixture.estate_account)).unwrap(),
        ))
        .as_principal(&specialist)
        .controls(BankMutationControls::new(
            scope.clone(),
            BankIdempotencyKey::new("owning-freeze-witness").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("exact construction: {cause:?}"));
    let Advance::Completed(done) = source.advance(&request) else {
        panic!("fresh native completion")
    };
    let Completion::Fresh { receipt, .. } = done.evidence() else {
        panic!("actual handler result")
    };
    assert!(
        receipt
            .clone()
            .take_performed_relational_product_change()
            .is_none(),
        "a descriptive clone cannot substitute publication authority"
    );
    let cancelled = WorthQueryCancellationSource::new();
    cancelled.cancel();
    let stopped_scope = WorthQueryRequestScope::new(None, cancelled.token());
    let stopped = fixture.world.runtime.request(&specialist, &stopped_scope);
    let (done, _denial) = done
        .close_with_evidence(&stopped)
        .err()
        .expect("native close refusal returns unchanged completion");
    let (_closed, completion) = done
        .close_with_evidence(&request)
        .unwrap_or_else(|(_, cause)| panic!("fresh lawful close: {cause:?}"));
    let Completion::Fresh {
        mut receipt,
        result,
        ..
    } = completion
    else {
        panic!("native provenance survived refusal")
    };
    assert_eq!(result.estate, fixture.estate);
    assert!(
        receipt.take_performed_relational_product_change().is_some(),
        "the original move-only performed witness crosses only lawful close"
    );
    assert!(
        receipt.take_performed_relational_product_change().is_none(),
        "the original witness transfers once"
    );
}

#[test]
fn eight_remaining_source_actions_install_exact_completion_and_deployment_meaning() {
    use bank_domain::schema::*;
    use worth_query_host::facade::declaration::application_program::*;
    fn verify<P: ApplicationProgramDefinition<BankSchema>>(notify: bool) {
        let program = ApplicationProgramAuthoring::<BankSchema, P>::begin()
            .validated_program()
            .unwrap();
        let expected_selectors = [
            <bank_server::DeathNotificationAction as ApplicationAction<
                BankSchema,
                NotifyEstateDeathMutationBinding,
            >>::selector(),
            <bank_server::EstateAccountFreezeAction as ApplicationAction<
                BankSchema,
                FreezeEstateAccountMutationBinding,
            >>::selector(),
            <bank_server::EstateCaseOpeningAction as ApplicationAction<
                BankSchema,
                OpenEstateCaseMutationBinding,
            >>::selector(),
            <bank_server::ExecutorRecognitionAction as ApplicationAction<
                BankSchema,
                RecognizeEstateExecutorMutationBinding,
            >>::selector(),
            <bank_server::EstateReleaseAction as ApplicationAction<
                BankSchema,
                ReleaseEstateMutationBinding,
            >>::selector(),
            <bank_server::EstateDisbursementAction as ApplicationAction<
                BankSchema,
                DisburseEstateMutationBinding,
            >>::selector(),
            <bank_server::DeathNoticeRetransmissionAction as ApplicationAction<
                BankSchema,
                RetransmitEstateDeathNoticeMutationBinding,
            >>::selector(),
            <bank_server::ProgramDeploymentRevocationAction as ApplicationAction<
                BankSchema,
                ProgramDeploymentRevocationBinding,
            >>::selector(),
        ];
        for (index, selector) in expected_selectors.into_iter().enumerate() {
            let installed = program
                .actions()
                .iter()
                .filter(|candidate| candidate.binding() == selector.binding())
                .collect::<Vec<_>>();
            if index == 0 && !notify {
                assert!(installed.is_empty());
                continue;
            }
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
            assert_eq!(installed[0].delivery().unwrap(), selector.delivery());
            assert!(matches!(
                installed[0].delivery().unwrap(),
                ApplicationActionDeliveryDeclaration::SourceCompletion { .. }
            ));
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
    verify::<bank_server::BankApplication>(true);
    verify::<bank_server::BankApplicationP1>(false);
    verify::<bank_server::BankApplicationP2>(false);
}
