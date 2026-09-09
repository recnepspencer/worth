use crate::basis_lifecycle::BasisOperationLane;
use crate::ordinary::live::{open_installed_read_live, WorthQueryLiveOpenOutcome};
use crate::ordinary::read::WorthQueryReadJourneyCounters;
use crate::runtime::WorthQueryWorkspace;

use super::conditional_attempt::{
    evaluate_fresh_conditionals, WorthQueryConditionalPromotionOutcome,
    WorthQueryProjectionReadyToOpen,
};
use super::promotion_preflight::{
    admit_projection_promotion, WorthQueryProjectionPreflightOutcome,
};
use super::source::WorthQueryProjectionLifecycleSource;
use super::{
    WorthQueryCurrentDomainProjection, WorthQueryLiveBoundDomainProjection,
    WorthQueryLiveProjectionReceipt, WorthQueryProjectionPromotionCounters,
    WorthQueryProjectionPromotionDenialKind, WorthQueryProjectionPromotionOutcome,
    WorthQueryProjectionPromotionStop,
};

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryCurrentDomainProjection<D, O, F, L>
{
    pub fn promote(
        self,
        workspace: &mut WorthQueryWorkspace,
    ) -> WorthQueryProjectionPromotionOutcome<D, O, F, L> {
        let admitted = match admit_projection_promotion(self, workspace) {
            WorthQueryProjectionPreflightOutcome::Admitted(admitted) => admitted,
            WorthQueryProjectionPreflightOutcome::Stopped(outcome) => return *outcome,
        };
        let ready = match evaluate_fresh_conditionals(admitted, workspace) {
            WorthQueryConditionalPromotionOutcome::Ready(ready) => ready,
            WorthQueryConditionalPromotionOutcome::Stopped(outcome) => return *outcome,
        };
        open_managed_live_projection(ready, workspace)
    }
}

fn open_managed_live_projection<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>(
    ready: Box<WorthQueryProjectionReadyToOpen<D, O, F, L>>,
    workspace: &mut WorthQueryWorkspace,
) -> WorthQueryProjectionPromotionOutcome<D, O, F, L> {
    let WorthQueryProjectionReadyToOpen {
        current,
        read,
        mut counters,
        attempt,
        operational_identity,
        resource_name,
        conditional_provenance,
    } = *ready;
    match open_lifecycle_read(resource_name.clone(), read, workspace) {
        WorthQueryLiveOpenOutcome::Opened(completion) => {
            retain_journey_counters(&mut counters, completion.journey_counters());
            let read_context_identity = completion.context_receipt().digest().to_string();
            let handle = completion.into_handle();
            let closure = current
                .snapshot()
                .semantic_dependency_closure()
                .expect("settled installed projection retains its dependency closure");
            if let Err(error) = workspace.register_installed_live_route::<D, O, F>(&handle, closure)
            {
                let detail = error.to_string();
                let _ = handle.close_with_cause(
                    workspace,
                    crate::ordinary::live::WorthQueryManagedLiveCloseCause::ReplacementRollback,
                );
                return WorthQueryProjectionPromotionOutcome::Failed(
                    WorthQueryProjectionPromotionStop::new(
                        *current,
                        WorthQueryProjectionPromotionDenialKind::ManagedLiveOpen,
                        detail,
                        counters,
                    ),
                );
            }
            let settled_identity = current.settled.identity().to_string();
            let (settled, basis, predecessor_identity) = (*current).into_live_parts();
            let receipt = WorthQueryLiveProjectionReceipt::new(
                operational_identity,
                resource_name,
                settled_identity,
                attempt,
                read_context_identity,
                counters,
            );
            WorthQueryProjectionPromotionOutcome::Promoted(
                WorthQueryLiveBoundDomainProjection::mint(
                    settled,
                    basis,
                    predecessor_identity,
                    handle,
                    receipt,
                    conditional_provenance,
                ),
            )
        }
        WorthQueryLiveOpenOutcome::Stopped(stop) => {
            retain_journey_counters(&mut counters, stop.read_stop().journey_counters());
            WorthQueryProjectionPromotionOutcome::Failed(WorthQueryProjectionPromotionStop::new(
                *current,
                WorthQueryProjectionPromotionDenialKind::ManagedLiveOpen,
                format!("managed live open stopped at {:?}", stop.source()),
                counters,
            ))
        }
    }
}

pub(super) fn open_lifecycle_read(
    resource_name: String,
    read: crate::ordinary::read::WorthQueryReadDeclaration,
    workspace: &mut WorthQueryWorkspace,
) -> WorthQueryLiveOpenOutcome {
    open_installed_read_live(resource_name, read, workspace)
}

pub(super) fn retain_journey_counters(
    target: &mut WorthQueryProjectionPromotionCounters,
    source: &WorthQueryReadJourneyCounters,
) {
    target.context_admission_attempts = source.context_admission_attempt_count();
    target.planning_attempts = source.planning_attempt_count();
    target.planning_completions = source.planning_completed_count();
    target.lower_runtime_contacts = source.lower_runtime_execution_attempt_count();
    target.managed_resource_registrations = source.lower_runtime_execution_completed_count();
}
