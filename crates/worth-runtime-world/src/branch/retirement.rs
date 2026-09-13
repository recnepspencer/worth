/// Typed pre-effect denials for product-reference retirement. Retirement
/// never cascades into a component-owner branch deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldBranchRetirementDenial {
    OwnerUnavailable,
    UnknownBranch,
    AlreadyRetired,
    RetentionStillRequired,
    CapacityExhausted,
}
