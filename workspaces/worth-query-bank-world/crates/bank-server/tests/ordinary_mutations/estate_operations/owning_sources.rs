//! Genuine source custody and post-cleanup evidence, distinct from published effect recovery.
use super::freeze_account::fixture::{exact_freeze_world, revocable_freeze_world};
use crate::support::request_scope;
use bank_domain::{
    proposals::BankIdempotencyKey,
    schema::{AccountStatus, FreezeEstateAccount},
};
use bank_server::{BankMutationControls, BankReadControls, mutations, queries};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQueryApplicationRecoveryRequestDenial, WorthQueryApplicationRequestMutationDenial,
        WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
        WorthQuerySourceOnlyDenial, WorthQuerySourceOnlyPosture,
    },
};
#[test]
fn owning_freeze_partial_fresh_permission_recovery_and_issuer_cleanup() {
    let fixture = revocable_freeze_world("owning-freeze-partial");
    let source = {
        let specialist = fixture.authenticate_specialist();
        let scope = request_scope();
        let input = FreezeEstateAccount::try_from(fixture.action(fixture.estate_account)).unwrap();
        let source = fixture
            .world
            .runtime
            .mutate(mutations::freeze_estate_account(input))
            .as_principal(&specialist)
            .controls(BankMutationControls::new(
                scope.clone(),
                BankIdempotencyKey::new("owning-freeze-partial").unwrap(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact native construction: {cause:?}"));
        fixture.world.runtime.fail_next_durable_append_for_test();
        let request = fixture.world.runtime.request(&specialist, &scope);
        let Advance::Progress(source) = source.advance(&request) else {
            panic!("actual append partial")
        };
        assert_eq!(
            source.posture(),
            WorthQuerySourceOnlyPosture::ProductUnpublished
        );
        source
    };
    let source = {
        let foreign = fixture.authenticate_foreign_owner();
        let scope = request_scope();
        let request = fixture.world.runtime.request(&foreign, &scope);
        let source = match source.advance(&request) {
            Advance::Refused {
                operation,
                denial:
                    WorthQuerySourceOnlyDenial::Recovery(
                        WorthQueryApplicationRecoveryRequestDenial::Request(
                            WorthQueryApplicationRequestMutationDenial::Authorization(_),
                        ),
                    ),
            } => operation,
            _ => panic!("fresh foreign actor retains the real partial owner"),
        };
        assert_eq!(
            source.posture(),
            WorthQuerySourceOnlyPosture::ProductUnpublished
        );
        source
    };
    let specialist = fixture.authenticate_specialist();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&specialist, &scope);
    let Advance::Progress(source) = source.advance(&request) else {
        panic!("fresh capability recovers source")
    };
    let Advance::Completed(done) = source.advance(&request) else {
        panic!("original keyed publication evidence")
    };
    assert!(matches!(
        done.evidence(),
        Completion::Recovered {
            publication: Some(_),
            ..
        }
    ));
    let view = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&specialist)
        .controls(BankReadControls::current(request_scope(), 1, 20_000).unwrap())
        .execute()
        .unwrap();
    assert_eq!(view.rows()[0].account().status(), AccountStatus::Frozen);
    // Only after actual publication: no mutation is attempted against an unpublished held World.
    fixture.revoke_freeze_permission();
    let closed = done
        .close_with_evidence(&request)
        .unwrap_or_else(|(_, cause)| {
            panic!("issuer cleanup does not reacquire revoked business permission: {cause:?}")
        });
    let bank_receipt = bank_server::BankCommitReceipt::from_closed_source(closed);
    assert_eq!(bank_receipt.changed_record_count(), 2);
    let input = FreezeEstateAccount::try_from(fixture.action(fixture.estate_account)).unwrap();
    let retry = fixture
        .world
        .runtime
        .mutate(mutations::freeze_estate_account(input))
        .as_principal(&specialist)
        .controls(BankMutationControls::new(
            scope.clone(),
            BankIdempotencyKey::new("owning-freeze-partial").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("description is not permission: {cause:?}"));
    let Advance::Refused {
        operation: _retry_custody,
        denial:
            WorthQuerySourceOnlyDenial::Mutation(
                WorthQueryApplicationRequestMutationDenial::Authorization(_),
            ),
    } = retry.advance(&request)
    else {
        panic!("replay still requires the actual current capability")
    };
}
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
