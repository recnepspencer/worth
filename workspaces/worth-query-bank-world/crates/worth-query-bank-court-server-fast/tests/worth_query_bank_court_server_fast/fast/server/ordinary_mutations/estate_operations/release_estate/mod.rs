pub(crate) use worth_query_bank_worlds::facade::server::estate_operations::release_estate::*;

pub(crate) mod readiness_refusal;

#[test]
fn public_progression_releases_the_exact_ready_estate_and_recovers_retry() {
    let fixture = release_world("estate-release-commit", ReleaseWorldSpec::ready());
    let specialist = fixture.authenticate_actor();
    let binding = idempotency(11);
    let outcome = {
        let closing_actor = &specialist;
        let closing_scope = worth_query_bank_worlds::facade::server::support::request_scope();
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
        let closing_scope = worth_query_bank_worlds::facade::server::support::request_scope();
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
