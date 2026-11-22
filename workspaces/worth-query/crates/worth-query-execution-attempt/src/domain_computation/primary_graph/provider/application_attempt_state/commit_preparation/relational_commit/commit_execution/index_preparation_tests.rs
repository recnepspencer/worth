use super::*;

#[test]
fn expired_candidate_remains_a_typed_retryable_defer() {
    let stop = index_preparation_stop(
            worth_relational::facade::indexes::DerivedIndexMaintenanceDenial {
                kind: worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::CandidateLifetimeExpired {
                    maximum_lifetime_millis: 17,
                },
                work: Default::default(),
            },
        );
    let crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(deferred) = stop
    else {
        panic!("candidate expiry must remain retryable before World effect");
    };
    assert_eq!(
            deferred.kind(),
            crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind::CandidateLifetimeExpired {
                maximum_lifetime_millis: 17,
            },
        );
}

#[test]
fn cold_reconstruction_exhaustion_remains_a_typed_index_budget_denial() {
    let stop = index_preparation_stop(
            worth_relational::facade::indexes::DerivedIndexMaintenanceDenial {
                kind: worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired,
                work: Default::default(),
            },
        );
    let crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(denial) =
        stop
    else {
        panic!("cold index exhaustion must deny before World effect");
    };
    assert_eq!(
            denial.kind(),
            crate::domain_computation::WorthQueryProviderSessionDenialKind::IndexMaintenanceBudgetExceeded,
        );
}
