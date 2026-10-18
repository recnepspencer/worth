//! Real readiness and capability refusals retain their source owner.
use super::*;

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
        let (_denial_source, denial) = {
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = release(&fixture, &specialist, idempotency(31 + ordinal as u8)) else { panic!("native pre-effect refusal") };
            (
                original_source,
                cause
                    .downcast::<bank_server::BankEstateReleaseProjectionDenial>()
                    .expect("native projection cause"),
            )
        };
        assert_executor_denial(executor, denial);
        assert_release_posture(&fixture, EstateCaseStatus::Open);
    }
}

fn assert_executor_denial(executor: ExecutorPosture, denial: BankEstateReleaseProjectionDenial) {
    match executor {
        ExecutorPosture::Missing => assert!(
            matches!(
                denial,
                BankEstateReleaseProjectionDenial::ExecutorRelationCardinality { observed: 0 }
            ),
            "unexpected missing-executor denial: {denial:?}"
        ),
        ExecutorPosture::UnrecognizedAuthority => assert!(matches!(
            denial,
            BankEstateReleaseProjectionDenial::RecognizedExecutorAuthorityMissing
        )),
        ExecutorPosture::WrongHolderAuthority => assert!(matches!(
            denial,
            BankEstateReleaseProjectionDenial::LegalAuthorityHolderMismatch
        )),
        ExecutorPosture::WrongEstateAuthority => assert!(matches!(
            denial,
            BankEstateReleaseProjectionDenial::LegalAuthorityEstateMismatch
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
        let (_denial_source, denial) = {
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused { operation: original_source, denial: worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::Handler(worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Handler(cause))) } = release(&fixture, &specialist, idempotency(41 + ordinal as u8)) else { panic!("native pre-effect refusal") };
            (
                original_source,
                cause
                    .downcast::<bank_server::BankEstateReleaseProjectionDenial>()
                    .expect("native projection cause"),
            )
        };
        assert_review_denial(review, denial);
        assert_release_posture(&fixture, EstateCaseStatus::Open);
    }
}

fn assert_review_denial(review: ReviewPosture, denial: BankEstateReleaseProjectionDenial) {
    match review {
        ReviewPosture::Missing => {
            assert!(matches!(
                denial,
                BankEstateReleaseProjectionDenial::EntityResolution(_)
            ))
        }
        ReviewPosture::Required => assert!(matches!(
            denial,
            BankEstateReleaseProjectionDenial::ReleaseReviewIncomplete
        )),
        ReviewPosture::WrongKind => assert!(matches!(
            denial,
            BankEstateReleaseProjectionDenial::ReleaseReviewWrongKind
        )),
        ReviewPosture::Retargeted => assert!(matches!(
            denial,
            BankEstateReleaseProjectionDenial::ReviewEstateMismatch
        )),
        ReviewPosture::NoReviewer => assert!(matches!(
            denial,
            BankEstateReleaseProjectionDenial::ReviewPrincipalCardinality { observed: 0 }
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
        let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused {
            operation: _denial_source,
            denial:
                worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(
                    denial,
                ),
        } = release(&fixture, &actor, idempotency(51 + ordinal as u8))
        else {
            panic!("native pre-effect authorization refusal")
        };
        assert!(matches!(
            denial,
            ref denial
                if matches!(
                denial.kind(),
                worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::Authorization(_)
            )
        ));
        assert_release_posture(&fixture, EstateCaseStatus::Open);
    }
}
