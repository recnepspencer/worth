//! Finite history-capacity allowance for the directory-Home fixture model.
use super::identity;
use worth_query_host::facade::runtime::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    WorthQueryProductWorldResources,
};

pub(super) fn resources() -> WorthQueryProductWorldResources {
    // Checkpoint capture processes history as well as the current action. Allocate
    // the ordinary unit per declared retained slot, under the SAME original request.
    // This is fixture policy for the finite model, not a per-original cost quote:
    // retained occurrences do not bound the entire canonical prefix. Larger or
    // differently retained histories may still exhaust this finite allowance.
    let work = identity::RETAINED_COMPOSITE_COMMITS
        .checked_mul(identity::ORDINARY_REQUEST_WORK_ALLOCATION)
        .expect("the fixture's declared history work allowance fits its counter");
    identity::world_resources_with_policy(ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(work)),
    ))
}
