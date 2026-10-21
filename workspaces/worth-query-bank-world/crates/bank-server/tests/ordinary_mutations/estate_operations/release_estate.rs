#[path = "release_estate/fixture.rs"]
mod fixture;

use bank_domain::{
    estate::{EstateAction, EstateCaseStatus, LegalAuthorityId, MandatoryReviewId},
    model::BankPrincipalId,
    proposals::BankIdempotencyKey,
};
use bank_server::{
    queries, BankAuthenticatedPrincipal, BankCommitReceipt, BankEstateReleaseProjectionDenial,
    BankReadControls,
};

use self::fixture::{
    release_world, ActorConflict, ExecutorPosture, ReleaseFixture, ReleaseWorldSpec, ReviewPosture,
};
use crate::support::request_scope;

#[test]
fn public_progression_releases_the_exact_ready_estate_and_recovers_retry() {
    let fixture = release_world("estate-release-commit", ReleaseWorldSpec::ready());
    let specialist = fixture.authenticate_actor();
    let binding = idempotency(11);
    let outcome = {
        let closing_actor = &specialist;
        let closing_scope = crate::support::request_scope();
        let closing_request = fixture.world.runtime.request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = release(&fixture, &specialist, binding.clone())
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };

    let receipt = outcome;
    assert_eq!(receipt.changed_record_count(), 2);
    assert_eq!(receipt.emitted_effect_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(18));
    assert_admission_derives_the_request_identities_once(receipt.canonical_work());
    assert_release_posture(&fixture, EstateCaseStatus::Released);
    assert_equivalent_retry(&fixture, &specialist, binding, &receipt);

    let (_fresh_source, fresh) = {
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = release(&fixture, &specialist, idempotency(12)) else { panic!("native pre-effect refusal") };
        (
            original_source,
            cause
                .downcast::<bank_server::BankEstateReleaseProjectionDenial>()
                .expect("native projection cause"),
        )
    };
    assert!(matches!(
        fresh,
        BankEstateReleaseProjectionDenial::EstateNotOpen
    ));
}

#[test]
fn four_lawful_executors_and_many_unrelated_reviews_preserve_bounded_readiness() {
    let fixture = release_world(
        "estate-release-additional-truth",
        ReleaseWorldSpec {
            additional_executors: 3,
            unrelated_reviews: 64,
            ..ReleaseWorldSpec::ready()
        },
    );
    let specialist = fixture.authenticate_actor();
    let outcome = {
        let closing_actor = &specialist;
        let closing_scope = crate::support::request_scope();
        let closing_request = fixture.world.runtime.request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = release(&fixture, &specialist, idempotency(21))
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    let receipt = outcome;
    assert_eq!(receipt.decision_fact_count(), Some(18));
    assert_release_posture(&fixture, EstateCaseStatus::Released);
}

fn release(
    fixture: &ReleaseFixture,
    specialist: &BankAuthenticatedPrincipal,
    idempotency: BankIdempotencyKey,
) -> super::EstateSourceAdvance<bank_domain::schema::ReleaseEstate, bank_server::EstateReleaseAction>
{
    {
        let source_actor = specialist;
        let source_scope = &request_scope();
        let source_key = &idempotency;
        let source_intent = bank_domain::schema::ReleaseEstate::try_from(fixture.action())
            .expect("typed estate command preserves the exact variant");
        let source_request = fixture.world.runtime.request(source_actor, source_scope);
        let source_operation = fixture
            .world
            .runtime
            .mutate(bank_server::mutations::release_estate(source_intent))
            .as_principal(source_actor)
            .controls(bank_server::BankMutationControls::new(
                (*source_scope).clone(),
                (*source_key).clone(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
        source_operation.advance(&source_request)
    }
}

fn assert_equivalent_retry(
    fixture: &ReleaseFixture,
    specialist: &BankAuthenticatedPrincipal,
    binding: BankIdempotencyKey,
    committed: &BankCommitReceipt,
) {
    let retry = {
        let closing_actor = specialist;
        let closing_scope = crate::support::request_scope();
        let closing_request = (fixture)
            .world
            .runtime
            .request(closing_actor, &closing_scope);
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
            done,
        ) = release(fixture, specialist, binding.clone())
        else {
            panic!("native source completion")
        };
        assert!(matches!(
            done.evidence(),
            worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Recovered { .. }
        ));
        bank_server::BankCommitReceipt::from_closed_source(
            done.close_with_evidence(&closing_request)
                .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}")),
        )
    };
    let recovered = retry;
    assert_eq!(committed.aftermath(), recovered.aftermath());
    assert_admission_derives_the_request_identities_once(recovered.canonical_work());

    assert_release_witness_drift(fixture, specialist, &binding);
}

fn assert_release_witness_drift(
    fixture: &ReleaseFixture,
    specialist: &BankAuthenticatedPrincipal,
    binding: &BankIdempotencyKey,
) {
    let EstateAction::ReleaseEstate {
        estate,
        executor,
        authority,
        review,
    } = fixture.action()
    else {
        unreachable!("the release fixture must carry a release action")
    };
    let drifted_witnesses = [
        EstateAction::ReleaseEstate {
            estate,
            executor: BankPrincipalId::new(executor.get() + 100).unwrap(),
            authority,
            review,
        },
        EstateAction::ReleaseEstate {
            estate,
            executor,
            authority: LegalAuthorityId::new(authority.get() + 100).unwrap(),
            review,
        },
        EstateAction::ReleaseEstate {
            estate,
            executor,
            authority,
            review: MandatoryReviewId::new(review.get() + 100).unwrap(),
        },
    ];
    for action in drifted_witnesses {
        let _denial = {
            let source_actor = specialist;
            let source_scope = &request_scope();
            let source_key = binding;
            let source_intent = bank_domain::schema::ReleaseEstate::try_from(action)
                .expect("typed estate command preserves the exact variant");
            let source_request = fixture.world.runtime.request(source_actor, source_scope);
            let source_operation = fixture
                .world
                .runtime
                .mutate(bank_server::mutations::release_estate(source_intent))
                .as_principal(source_actor)
                .controls(bank_server::BankMutationControls::new(
                    (*source_scope).clone(),
                    (*source_key).clone(),
                ))
                .retain_operation()
                .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(
                source_terminal,
            ) = source_operation.advance(&source_request)
            else {
                panic!("exact keyed intent drift is terminal")
            };
            assert!(matches!(source_terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)));
            source_terminal
                .close(&source_request)
                .unwrap_or_else(|(_, cause)| panic!("lawful drift close: {cause:?}"))
        };
    }
}

fn assert_release_posture(fixture: &ReleaseFixture, expected: EstateCaseStatus) {
    let actor = fixture.authenticate_actor();
    let result = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&actor)
        .controls(BankReadControls::current(request_scope(), 1).unwrap())
        .execute()
        .expect("the assigned specialist should read the authoritative estate status");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].id(), fixture.estate);
    assert_eq!(result.rows()[0].status(), expected);
}

fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("release-estate-{identity}")).unwrap()
}

fn assert_admission_derives_the_request_identities_once(
    phases: bank_server::BankCommitCanonicalWorkPhases,
) {
    // Binding-selected capability admission governs its input as well as the mutation
    // input/key identities. All three native derivations are included here; later
    // publication and domain recovery do not rebuild these identities.
    let admission = phases.admission();
    assert_eq!(admission.basis_preparations(), 0);
    assert_eq!(admission.digest_derivations(), 3);
    assert_eq!(admission.canonical_entries(), 3);
    assert_eq!(admission.canonical_encoded_bytes(), 323);
    assert_eq!(admission.sha256_input_bytes(), 542);
    assert_eq!(admission.sha256_compression_blocks(), 11);
    assert_eq!(admission.digest_text_materializations(), 0);
    for work in [
        phases.installation(),
        phases.execution(),
        phases.provider_commit(),
        phases.projection(),
        phases.retry_resolution(),
        phases.publication(),
    ] {
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 0);
        assert_eq!(work.canonical_encoded_bytes(), 0);
        assert_eq!(work.digest_text_materializations(), 0);
    }
}

#[path = "release_estate/readiness_refusal.rs"]
mod readiness_refusal;
