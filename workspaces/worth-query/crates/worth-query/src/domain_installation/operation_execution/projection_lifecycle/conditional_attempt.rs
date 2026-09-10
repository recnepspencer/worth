use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryWorkspace;

use super::promotion_preflight::WorthQueryAdmittedProjectionPromotion;
use super::{
    WorthQueryCurrentDomainProjection, WorthQueryProjectionPromotionCounters,
    WorthQueryProjectionPromotionDenialKind, WorthQueryProjectionPromotionOutcome,
    WorthQueryProjectionPromotionStop,
};

pub(super) struct WorthQueryProjectionReadyToOpen<D, O, F, L: BasisOperationLane> {
    pub(super) current: Box<WorthQueryCurrentDomainProjection<D, O, F, L>>,
    pub(super) read: crate::ordinary::read::WorthQueryReadDeclaration,
    pub(super) counters: WorthQueryProjectionPromotionCounters,
    pub(super) attempt: u64,
    pub(super) operational_identity: String,
    pub(super) resource_name: String,
    pub(super) conditional_provenance:
        Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
}

pub(super) enum WorthQueryConditionalPromotionOutcome<D, O, F, L: BasisOperationLane> {
    Ready(Box<WorthQueryProjectionReadyToOpen<D, O, F, L>>),
    Stopped(Box<WorthQueryProjectionPromotionOutcome<D, O, F, L>>),
}

pub(super) fn evaluate_fresh_conditionals<D, O, F, L: BasisOperationLane>(
    admitted: WorthQueryAdmittedProjectionPromotion<D, O, F, L>,
    workspace: &mut WorthQueryWorkspace,
) -> WorthQueryConditionalPromotionOutcome<D, O, F, L> {
    let WorthQueryAdmittedProjectionPromotion {
        current,
        read,
        counters,
    } = admitted;
    let ready = match super::conditional_core::evaluate_fresh_lifecycle_conditionals(
        &current.settled,
        workspace,
        counters,
        "worth_query_live_bound_projection_v1",
    ) {
        Ok(ready) => ready,
        Err(stop) => {
            let wrap = match stop.class {
                super::conditional_core::WorthQueryLifecycleConditionalStopClass::Deferred => {
                    WorthQueryProjectionPromotionOutcome::Deferred
                }
                super::conditional_core::WorthQueryLifecycleConditionalStopClass::Denied => {
                    WorthQueryProjectionPromotionOutcome::Denied
                }
                super::conditional_core::WorthQueryLifecycleConditionalStopClass::Failed => {
                    WorthQueryProjectionPromotionOutcome::Failed
                }
            };
            return stopped(current, wrap, stop.kind, stop.detail, stop.counters);
        }
    };
    WorthQueryConditionalPromotionOutcome::Ready(Box::new(WorthQueryProjectionReadyToOpen {
        current: Box::new(current),
        read,
        counters: ready.counters,
        attempt: ready.attempt,
        operational_identity: ready.operational_identity,
        resource_name: ready.resource_name,
        conditional_provenance: ready.conditional_provenance,
    }))
}

fn stopped<D, O, F, L: BasisOperationLane>(
    current: WorthQueryCurrentDomainProjection<D, O, F, L>,
    wrap: fn(
        WorthQueryProjectionPromotionStop<D, O, F, L>,
    ) -> WorthQueryProjectionPromotionOutcome<D, O, F, L>,
    kind: WorthQueryProjectionPromotionDenialKind,
    detail: String,
    counters: WorthQueryProjectionPromotionCounters,
) -> WorthQueryConditionalPromotionOutcome<D, O, F, L> {
    WorthQueryConditionalPromotionOutcome::Stopped(Box::new(wrap(
        WorthQueryProjectionPromotionStop::new(current, kind, detail, counters),
    )))
}
