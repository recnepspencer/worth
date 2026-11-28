/// Typed pre-effect denials for product-reference retirement. Retirement
/// never cascades into a component-owner branch deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldBranchRetirementDenial {
    StaleIncarnation,
    OwnerUnavailable,
    AlreadyRetired,
    CapacityExhausted,
}

use worth_execution::report::Disposition::*;
use worth_execution::report::RetryHint::WithFreshAuthority;

crate::denial_report::world_denial_report!(RuntimeWorldBranchRetirementDenial, "branch_retirement", {
    Self::StaleIncarnation => ("stale_incarnation", Conflict, Retry(WithFreshAuthority), "The branch selection belongs to a stale incarnation."),
    Self::OwnerUnavailable => ("owner_unavailable", Unavailable, Retry(WithFreshAuthority), "The World owner is unavailable."),
    Self::AlreadyRetired => ("already_retired", Conflict, Final, "The product branch is already retired."),
    Self::CapacityExhausted => ("capacity_exhausted", Exhausted, Final, "The World branch retirement capacity is exhausted."),
});

impl std::error::Error for RuntimeWorldBranchRetirementDenial {}
