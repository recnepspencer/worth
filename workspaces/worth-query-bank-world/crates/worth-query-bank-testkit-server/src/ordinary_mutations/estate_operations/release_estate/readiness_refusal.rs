//! Real readiness and capability refusals retain their source owner.
pub use super::*;

pub fn assert_executor_denial(
    executor: ExecutorPosture,
    denial: BankEstateReleaseProjectionDenial,
) {
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

pub fn assert_review_denial(review: ReviewPosture, denial: BankEstateReleaseProjectionDenial) {
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
