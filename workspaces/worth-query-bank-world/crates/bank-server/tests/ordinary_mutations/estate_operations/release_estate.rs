#[path = "release_estate/fixture.rs"]
mod fixture;

use bank_domain::{
    estate::{EstateAction, EstateCaseStatus, LegalAuthorityId, MandatoryReviewId},
    model::BankPrincipalId,
};
use bank_server::{
    queries, BankAuthenticatedPrincipal, BankCommitDenialKind, BankCommitDenialStage,
    BankCommitReceipt, BankEstateProgressionDenial, BankEstateReleaseProjectionDenial,
    BankMutationCommitOutcome, BankReadControls,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;

use self::fixture::{
    release_world, ActorConflict, ExecutorPosture, ReleaseFixture, ReleaseWorldSpec, ReviewPosture,
};
use crate::support::request_scope;

#[test]
fn public_progression_releases_the_exact_ready_estate_and_recovers_retry() {
    let fixture = release_world("estate-release-commit", ReleaseWorldSpec::ready());
    let specialist = fixture.authenticate_actor();
    let binding = idempotency(11);
    let outcome = release(&fixture, &specialist, binding)
        .expect("the exact ready estate should release through Query");

    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the first exact release must commit: {outcome:?}");
    };
    assert_eq!(receipt.changed_record_count(), 2);
    assert_eq!(receipt.emitted_effect_count(), 0);
    assert_eq!(receipt.decision_fact_count(), Some(16));
    assert_zero_canonical_work(receipt.canonical_work());
    assert_release_posture(&fixture, EstateCaseStatus::Released);
    assert_equivalent_retry(&fixture, &specialist, binding, &receipt);

    let fresh = release(&fixture, &specialist, idempotency(12))
        .expect_err("a fresh intent cannot release an already released estate");
    assert!(matches!(
        fresh,
        BankEstateProgressionDenial::EstateReleaseProjection(
            BankEstateReleaseProjectionDenial::EstateNotOpen
        )
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
    let outcome = release(&fixture, &specialist, idempotency(21))
        .expect("lawful additional executor and review truth must remain admissible");
    let BankMutationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the additional-truth release must commit: {outcome:?}");
    };
    assert_eq!(receipt.decision_fact_count(), Some(16));
    assert_release_posture(&fixture, EstateCaseStatus::Released);
}

#[test]
fn executor_must_have_exact_current_recognized_authority() {
    for (ordinal, executor) in [
        ExecutorPosture::Missing,
        ExecutorPosture::UnrecognizedAuthority,
        ExecutorPosture::WrongHolderAuthority,
        ExecutorPosture::WrongEstateAuthority,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = release_world(
            &format!("estate-release-executor-{executor:?}"),
            ReleaseWorldSpec {
                executor,
                ..ReleaseWorldSpec::ready()
            },
        );
        let specialist = fixture.authenticate_actor();
        let denial = release(&fixture, &specialist, idempotency(31 + ordinal as u8))
            .expect_err("an estate without a recognized exact executor must not release");
        assert_executor_denial(executor, denial);
        assert_release_posture(&fixture, EstateCaseStatus::Open);
    }
}

fn assert_executor_denial(executor: ExecutorPosture, denial: BankEstateProgressionDenial) {
    match executor {
        ExecutorPosture::Missing => assert!(
            matches!(
                denial,
                BankEstateProgressionDenial::EstateReleaseProjection(
                    BankEstateReleaseProjectionDenial::ExecutorRelationCardinality { observed: 0 }
                )
            ),
            "unexpected missing-executor denial: {denial:?}"
        ),
        ExecutorPosture::UnrecognizedAuthority => assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::RecognizedExecutorAuthorityMissing
            )
        )),
        ExecutorPosture::WrongHolderAuthority => assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::LegalAuthorityHolderMismatch
            )
        )),
        ExecutorPosture::WrongEstateAuthority => assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::LegalAuthorityEstateMismatch
            )
        )),
        ExecutorPosture::Ready => panic!("ready executor unexpectedly denied: {denial:?}"),
    }
}

#[test]
fn exact_completed_release_review_is_required() {
    for (ordinal, review) in [
        ReviewPosture::Missing,
        ReviewPosture::Required,
        ReviewPosture::WrongKind,
        ReviewPosture::Retargeted,
        ReviewPosture::NoReviewer,
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = release_world(
            &format!("estate-release-review-{review:?}"),
            ReleaseWorldSpec {
                review,
                ..ReleaseWorldSpec::ready()
            },
        );
        let specialist = fixture.authenticate_actor();
        let denial = release(&fixture, &specialist, idempotency(41 + ordinal as u8))
            .expect_err("a missing or invalid release review must deny before mutation");
        assert_review_denial(review, denial);
        assert_release_posture(&fixture, EstateCaseStatus::Open);
    }
}

fn assert_review_denial(review: ReviewPosture, denial: BankEstateProgressionDenial) {
    match review {
        ReviewPosture::Missing => {
            assert!(matches!(
                denial,
                BankEstateProgressionDenial::EstateReleaseProjection(
                    BankEstateReleaseProjectionDenial::EntityResolution(_)
                )
            ))
        }
        ReviewPosture::Required => assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::ReleaseReviewIncomplete
            )
        )),
        ReviewPosture::WrongKind => assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::ReleaseReviewWrongKind
            )
        )),
        ReviewPosture::Retargeted => assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::ReviewEstateMismatch
            )
        )),
        ReviewPosture::NoReviewer => assert!(matches!(
            denial,
            BankEstateProgressionDenial::EstateReleaseProjection(
                BankEstateReleaseProjectionDenial::ReviewPrincipalCardinality { observed: 0 }
            )
        )),
        ReviewPosture::Completed => panic!("completed review unexpectedly denied: {denial:?}"),
    }
}

#[test]
fn beneficiary_and_executor_callers_deny_at_capability_composition() {
    for (ordinal, actor_conflict) in [ActorConflict::Beneficiary, ActorConflict::Executor]
        .into_iter()
        .enumerate()
    {
        let fixture = release_world(
            &format!("estate-release-conflict-{actor_conflict:?}"),
            ReleaseWorldSpec {
                actor_conflict,
                ..ReleaseWorldSpec::ready()
            },
        );
        let actor = fixture.authenticate_actor();
        let denial = release(&fixture, &actor, idempotency(51 + ordinal as u8))
            .expect_err("conflicted release authority must deny before invariant projection");
        assert!(matches!(
            denial,
            BankEstateProgressionDenial::Authorization(_)
        ));
        assert_release_posture(&fixture, EstateCaseStatus::Open);
    }
}

fn release(
    fixture: &ReleaseFixture,
    specialist: &BankAuthenticatedPrincipal,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> Result<BankMutationCommitOutcome, BankEstateProgressionDenial> {
    fixture.world.runtime.release_estate(
        specialist,
        fixture.action(),
        idempotency,
        &request_scope(),
    )
}

fn assert_equivalent_retry(
    fixture: &ReleaseFixture,
    specialist: &BankAuthenticatedPrincipal,
    binding: WorthQueryApplicationIdempotencyBinding,
    committed: &BankCommitReceipt,
) {
    let retry = release(fixture, specialist, binding)
        .expect("equivalent authorized retry must resolve before released poststate");
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("the retry must recover its exact commit: {retry:?}");
    };
    assert_eq!(committed.aftermath(), recovered.aftermath());

    assert_raw_intent_drift(fixture, specialist, binding);
    assert_release_witness_drift(fixture, specialist, binding);
}

fn assert_raw_intent_drift(
    fixture: &ReleaseFixture,
    specialist: &BankAuthenticatedPrincipal,
    binding: WorthQueryApplicationIdempotencyBinding,
) {
    let drift = fixture
        .world
        .runtime
        .release_estate(
            specialist,
            fixture.action(),
            WorthQueryApplicationIdempotencyBinding::new(*binding.key_identity(), [99; 32]),
            &request_scope(),
        )
        .expect("intent drift remains a typed commit outcome");
    assert!(matches!(
        drift,
        BankMutationCommitOutcome::Denied {
            kind: BankCommitDenialKind::IdempotencyIntentDrift,
            stage: BankCommitDenialStage::Idempotency,
        }
    ));
}

fn assert_release_witness_drift(
    fixture: &ReleaseFixture,
    specialist: &BankAuthenticatedPrincipal,
    binding: WorthQueryApplicationIdempotencyBinding,
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
        let outcome = fixture
            .world
            .runtime
            .release_estate(specialist, action, binding, &request_scope())
            .expect("witness drift remains a typed idempotency outcome");
        assert!(matches!(
            outcome,
            BankMutationCommitOutcome::Denied {
                kind: BankCommitDenialKind::IdempotencyIntentDrift,
                stage: BankCommitDenialStage::Idempotency,
            }
        ));
    }
}

fn assert_release_posture(fixture: &ReleaseFixture, expected: EstateCaseStatus) {
    let actor = fixture.authenticate_actor();
    let result = fixture
        .world
        .runtime
        .query(queries::estate_case(fixture.estate))
        .as_principal(&actor)
        .controls(BankReadControls::current(request_scope(), 1, 20_000).unwrap())
        .execute()
        .expect("the assigned specialist should read the authoritative estate status");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].id(), fixture.estate);
    assert_eq!(result.rows()[0].status(), expected);
}

fn idempotency(identity: u8) -> WorthQueryApplicationIdempotencyBinding {
    WorthQueryApplicationIdempotencyBinding::new([identity; 32], [identity + 1; 32])
}

fn assert_zero_canonical_work(phases: bank_server::BankCommitCanonicalWorkPhases) {
    for work in [
        phases.installation(),
        phases.admission(),
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
