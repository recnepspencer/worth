use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryExecutableDomainOperation;
use crate::ordinary::live::WorthQueryLiveOpenOutcome;

use super::conditional_core::{
    evaluate_fresh_lifecycle_conditionals, WorthQueryLifecycleConditionalStopClass,
};
use super::promotion::{open_lifecycle_read, retain_journey_counters};
use super::promotion_preflight::{admit_projection_promotion_core, WorthQueryProjectionCoreStop};
use super::source::WorthQueryProjectionLifecycleSource;
use super::{
    WorthQueryCurrentWorkflowProjection, WorthQueryLiveBoundWorkflowProjection,
    WorthQueryLiveProjectionReceipt, WorthQueryWorkflowProjectionPromotionOutcome,
    WorthQueryWorkflowProjectionPromotionStop,
};

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryCurrentWorkflowProjection<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F>,
{
    pub fn promote(
        self,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> WorthQueryWorkflowProjectionPromotionOutcome<D, O, F, L> {
        let admitted =
            match admit_projection_promotion_core(&self.settled, self.lifecycle_basis(), workspace)
            {
                Ok(admitted) => admitted,
                Err(stop) => return map_preflight_stop(self, stop),
            };
        let ready = match evaluate_fresh_lifecycle_conditionals(
            &self.settled,
            workspace,
            admitted.counters,
            "worth_query_live_bound_workflow_projection_v1",
        ) {
            Ok(ready) => ready,
            Err(stop) => {
                let stopped = WorthQueryWorkflowProjectionPromotionStop::new(
                    self,
                    stop.kind,
                    stop.detail,
                    stop.counters,
                );
                return match stop.class {
                    WorthQueryLifecycleConditionalStopClass::Deferred => {
                        WorthQueryWorkflowProjectionPromotionOutcome::Deferred(stopped)
                    }
                    WorthQueryLifecycleConditionalStopClass::Denied => {
                        WorthQueryWorkflowProjectionPromotionOutcome::Denied(stopped)
                    }
                    WorthQueryLifecycleConditionalStopClass::Failed => {
                        WorthQueryWorkflowProjectionPromotionOutcome::Failed(stopped)
                    }
                };
            }
        };
        let resource_name = ready.resource_name.clone();
        match open_lifecycle_read(resource_name.clone(), admitted.read, workspace) {
            WorthQueryLiveOpenOutcome::Opened(completion) => {
                let mut counters = ready.counters;
                retain_journey_counters(&mut counters, completion.journey_counters());
                let read_context_identity = completion.context_receipt().digest().to_string();
                let handle = completion.into_handle();
                let closure = self
                    .settled
                    .semantic_dependency_closure()
                    .expect("settled workflow projection retains its dependency closure");
                if let Err(error) = workspace.register_installed_live_route(&handle, closure) {
                    let detail = error.to_string();
                    let _ = handle.close_with_cause(
                        workspace,
                        crate::ordinary::live::WorthQueryManagedLiveCloseCause::ReplacementRollback,
                    );
                    return WorthQueryWorkflowProjectionPromotionOutcome::Failed(
                        WorthQueryWorkflowProjectionPromotionStop::new(
                            self,
                            super::WorthQueryProjectionPromotionDenialKind::ManagedLiveOpen,
                            detail,
                            counters,
                        ),
                    );
                }
                let settled_identity = self.settled.identity().to_string();
                let (settled, basis, predecessor_identity) = self.into_live_parts();
                let receipt = WorthQueryLiveProjectionReceipt::new(
                    ready.operational_identity,
                    resource_name,
                    settled_identity,
                    ready.attempt,
                    read_context_identity,
                    counters,
                );
                WorthQueryWorkflowProjectionPromotionOutcome::Promoted(
                    WorthQueryLiveBoundWorkflowProjection::mint(
                        settled,
                        basis,
                        predecessor_identity,
                        handle,
                        receipt,
                        ready.conditional_provenance,
                    ),
                )
            }
            WorthQueryLiveOpenOutcome::Stopped(stop) => {
                let mut counters = ready.counters;
                retain_journey_counters(&mut counters, stop.read_stop().journey_counters());
                WorthQueryWorkflowProjectionPromotionOutcome::Failed(
                    WorthQueryWorkflowProjectionPromotionStop::new(
                        self,
                        super::WorthQueryProjectionPromotionDenialKind::ManagedLiveOpen,
                        format!("managed live open stopped at {:?}", stop.source()),
                        counters,
                    ),
                )
            }
        }
    }
}

fn map_preflight_stop<D, O, F, L: BasisOperationLane>(
    current: WorthQueryCurrentWorkflowProjection<D, O, F, L>,
    stop: WorthQueryProjectionCoreStop,
) -> WorthQueryWorkflowProjectionPromotionOutcome<D, O, F, L> {
    match stop {
        WorthQueryProjectionCoreStop::Stale(counters) => {
            WorthQueryWorkflowProjectionPromotionOutcome::Stale(current.into_stale(counters))
        }
        WorthQueryProjectionCoreStop::RebindRequired(counters) => {
            WorthQueryWorkflowProjectionPromotionOutcome::RebindRequired(
                current.into_rebind_required(counters),
            )
        }
        WorthQueryProjectionCoreStop::AuthorityRevalidationRequired(counters) => {
            WorthQueryWorkflowProjectionPromotionOutcome::AuthorityRevalidationRequired(
                current.into_authority_revalidation(counters),
            )
        }
        WorthQueryProjectionCoreStop::Denied {
            kind,
            detail,
            counters,
        } => WorthQueryWorkflowProjectionPromotionOutcome::Denied(
            WorthQueryWorkflowProjectionPromotionStop::new(current, kind, detail, counters),
        ),
    }
}
