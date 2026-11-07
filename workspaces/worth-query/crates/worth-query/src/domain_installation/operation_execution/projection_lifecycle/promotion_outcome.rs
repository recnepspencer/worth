use crate::basis_lifecycle::BasisOperationLane;

use super::{
    WorthQueryAuthorityRevalidationDomainProjection, WorthQueryLiveBoundDomainProjection,
    WorthQueryProjectionPromotionStop, WorthQueryRebindRequiredDomainProjection,
    WorthQueryStaleReadableDomainProjection,
};

#[must_use = "promotion outcomes retain either the managed resource or the prior projection state"]
pub enum WorthQueryProjectionPromotionOutcome<D, O, F, L: BasisOperationLane> {
    Promoted(WorthQueryLiveBoundDomainProjection<D, O, F, L>),
    Denied(WorthQueryProjectionPromotionStop<D, O, F, L>),
    Deferred(WorthQueryProjectionPromotionStop<D, O, F, L>),
    Stale(WorthQueryStaleReadableDomainProjection<D, O, F, L>),
    RebindRequired(WorthQueryRebindRequiredDomainProjection<D, O, F, L>),
    AuthorityRevalidationRequired(WorthQueryAuthorityRevalidationDomainProjection<D, O, F, L>),
    Failed(WorthQueryProjectionPromotionStop<D, O, F, L>),
}
#[cfg(test)]
#[path = "promotion_outcome_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
