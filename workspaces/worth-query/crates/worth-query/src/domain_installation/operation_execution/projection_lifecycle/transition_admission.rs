use crate::basis_lifecycle::BasisOperationLane;
use crate::ordinary::live::{WorthQueryLiveOpenOutcome, WorthQueryManagedLiveHandle};
use crate::runtime::WorthQueryWorkspace;

use super::conditional_core::{
    evaluate_fresh_lifecycle_conditionals, WorthQueryLifecycleConditionalStopClass,
};
use super::promotion::{open_lifecycle_read, retain_journey_counters};
use super::promotion_preflight::{admit_projection_promotion_core, WorthQueryProjectionCoreStop};
use super::{
    WorthQueryCurrentDomainProjection, WorthQueryCurrentWorkflowProjection,
    WorthQueryProjectionPromotionCounters, WorthQueryProjectionTransitionDenialKind,
    WorthQueryProjectionTransitionWork,
};

pub(super) trait WorthQueryLifecycleTransitionCandidate<D, O, F, L>: Sized
where
    L: BasisOperationLane,
{
    type Source: super::source::WorthQueryProjectionLifecycleSource<D, O, F, L>;

    fn source(&self) -> &Self::Source;
    fn lifecycle_basis(&self) -> &super::states::WorthQueryProjectionLifecycleBasis<L>;
}

impl<D, O, F, L: BasisOperationLane> WorthQueryLifecycleTransitionCandidate<D, O, F, L>
    for WorthQueryCurrentDomainProjection<D, O, F, L>
{
    type Source = crate::domain_installation::WorthQuerySettledDomainProjection<D, O, F, L>;

    fn source(&self) -> &Self::Source {
        &self.settled
    }

    fn lifecycle_basis(&self) -> &super::states::WorthQueryProjectionLifecycleBasis<L> {
        self.lifecycle_basis()
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryLifecycleTransitionCandidate<D, O, F, L>
    for WorthQueryCurrentWorkflowProjection<D, O, F, L>
{
    type Source = crate::domain_installation::WorthQuerySettledWorkflowProjection<D, O, F, L>;

    fn source(&self) -> &Self::Source {
        &self.settled
    }

    fn lifecycle_basis(&self) -> &super::states::WorthQueryProjectionLifecycleBasis<L> {
        self.lifecycle_basis()
    }
}

pub(super) struct WorthQueryAdmittedTransitionSuccessor<C> {
    pub(super) current: C,
    pub(super) handle: WorthQueryManagedLiveHandle,
    pub(super) ready: super::conditional_core::WorthQueryLifecycleConditionalCoreReady,
    pub(super) read_context_identity: String,
    pub(super) work: WorthQueryProjectionTransitionWork,
}

pub(super) struct WorthQueryTransitionSuccessorStop<C> {
    pub(super) current: C,
    pub(super) kind: WorthQueryProjectionTransitionDenialKind,
    pub(super) detail: String,
    pub(super) work: WorthQueryProjectionTransitionWork,
}

pub(super) fn open_transition_successor<D: 'static, O, F, L, C>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    current: C,
    workspace: &mut WorthQueryWorkspace,
    identity_family: &'static str,
    mut work: WorthQueryProjectionTransitionWork,
) -> Result<WorthQueryAdmittedTransitionSuccessor<C>, WorthQueryTransitionSuccessorStop<C>>
where
    L: BasisOperationLane,
    C: WorthQueryLifecycleTransitionCandidate<D, O, F, L>,
{
    let admitted = match admit_projection_promotion_core(
        current.source(),
        current.lifecycle_basis(),
        workspace,
    ) {
        Ok(admitted) => admitted,
        Err(stop) => {
            let (kind, detail, counters) = preflight_stop(stop);
            work.retain_candidate(counters);
            return Err(WorthQueryTransitionSuccessorStop {
                current,
                kind,
                detail,
                work,
            });
        }
    };
    let mut ready = match evaluate_fresh_lifecycle_conditionals(
        execution,
        current.source(),
        workspace,
        admitted.counters,
        identity_family,
    ) {
        Ok(ready) => ready,
        Err(stop) => {
            let kind = match stop.class {
                WorthQueryLifecycleConditionalStopClass::Deferred => {
                    WorthQueryProjectionTransitionDenialKind::ConditionalDeferred
                }
                WorthQueryLifecycleConditionalStopClass::Denied => {
                    WorthQueryProjectionTransitionDenialKind::ConditionalDenied
                }
                WorthQueryLifecycleConditionalStopClass::Failed => {
                    WorthQueryProjectionTransitionDenialKind::ConditionalFailed
                }
            };
            work.retain_candidate(stop.counters);
            return Err(WorthQueryTransitionSuccessorStop {
                current,
                kind,
                detail: stop.detail,
                work,
            });
        }
    };
    let resource_name = ready.resource_name.clone();
    match open_lifecycle_read(resource_name, admitted.read, workspace) {
        WorthQueryLiveOpenOutcome::Opened(completion) => {
            let mut counters = ready.counters;
            retain_journey_counters(&mut counters, completion.journey_counters());
            ready.counters = counters;
            work.retain_candidate(counters);
            Ok(WorthQueryAdmittedTransitionSuccessor {
                current,
                read_context_identity: completion.context_receipt().digest().to_string(),
                handle: completion.into_handle(),
                ready,
                work,
            })
        }
        WorthQueryLiveOpenOutcome::Stopped(stop) => {
            let mut counters = ready.counters;
            retain_journey_counters(&mut counters, stop.read_stop().journey_counters());
            work.retain_candidate(counters);
            Err(WorthQueryTransitionSuccessorStop {
                current,
                kind: WorthQueryProjectionTransitionDenialKind::ManagedLiveOpen,
                detail: format!("managed live open stopped at {:?}", stop.source()),
                work,
            })
        }
    }
}

fn preflight_stop(
    stop: WorthQueryProjectionCoreStop,
) -> (
    WorthQueryProjectionTransitionDenialKind,
    String,
    WorthQueryProjectionPromotionCounters,
) {
    match stop {
        WorthQueryProjectionCoreStop::Stale(counters) => (
            WorthQueryProjectionTransitionDenialKind::CandidateStale,
            "candidate installation generation is stale".into(),
            counters,
        ),
        WorthQueryProjectionCoreStop::RebindRequired(counters) => (
            WorthQueryProjectionTransitionDenialKind::CandidateRebindRequired,
            "candidate package identity requires explicit rebind".into(),
            counters,
        ),
        WorthQueryProjectionCoreStop::AuthorityRevalidationRequired(counters) => (
            WorthQueryProjectionTransitionDenialKind::CandidateAuthorityRevalidationRequired,
            "candidate basis requires authority revalidation".into(),
            counters,
        ),
        WorthQueryProjectionCoreStop::Denied {
            kind,
            detail,
            counters,
        } => (
            WorthQueryProjectionTransitionDenialKind::CandidatePromotion(kind),
            detail.into(),
            counters,
        ),
    }
}
