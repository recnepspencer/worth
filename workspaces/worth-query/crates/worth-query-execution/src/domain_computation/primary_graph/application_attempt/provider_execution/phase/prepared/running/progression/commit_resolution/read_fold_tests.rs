use super::*;
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Denial;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolStage as Stage,
};
#[test]
fn read_fold_indeterminate_idempotency_stop_replaces_the_prior_failure() {
    let prior = WorthQueryProviderSessionFailure::new(
        Kind::ProviderRejected,
        Stage::Commit,
        "commit reply never arrived",
        Default::default(),
    );
    let evidence = || {
        WorthQueryApplicationUnresolvedCommitEvidence::from_provider_session_failure(
            WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired,
            &prior,
        )
    };
    for stop in [Err(Control::Cancelled), Err(Control::TimedOut), Ok(Kind::ExecutionResource { denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None })] {
        let WorthQueryProviderProgressionOutcome::Indeterminate(actual) = indeterminate_read_denied(evidence(), Denial::ExecutionDenied(stop)) else { panic!("the original commit still needs recovery") };
        assert_eq!(actual.denial_kind(), stop);
        assert_eq!(actual.recovery(), WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired);
        assert_eq!(actual.stage(), Stage::Commit);
    }
    let original = evidence();
    let WorthQueryProviderProgressionOutcome::Indeterminate(actual) =
        indeterminate_read_denied(original.clone(), Denial::Unavailable)
    else {
        panic!("ordinary unavailable preserves recovery")
    };
    assert_eq!(actual, original);
}
