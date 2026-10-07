use super::PhysicalCurrentRootAdvanceFailureCause;
use crate::physical_runtime::RootNamespaceDurablePhysicalMutationMembers;
use worth_store_physical_format::DurablePhysicalRootManifest;

pub(super) fn validate_advance(
    current_root: &DurablePhysicalRootManifest,
    durable: &RootNamespaceDurablePhysicalMutationMembers,
) -> Option<PhysicalCurrentRootAdvanceFailureCause> {
    if current_root != durable.source_root() {
        return Some(PhysicalCurrentRootAdvanceFailureCause::CurrentRootMismatch);
    }
    if current_root.tier_epoch_anchor() != durable.candidate_tier_epoch_anchor() {
        return Some(PhysicalCurrentRootAdvanceFailureCause::TransitionIdentityMismatch);
    }
    if !durable.transition_matches() {
        return Some(PhysicalCurrentRootAdvanceFailureCause::TransitionIdentityMismatch);
    }
    let identity = durable.identity();
    let group = durable.group_basis();
    if !identity.matches_group(group, durable.members().len()) {
        return Some(PhysicalCurrentRootAdvanceFailureCause::TransitionIdentityMismatch);
    }
    if identity.source_generation() != current_root.generation()
        || identity.candidate_generation() != durable.current_root_generation()
    {
        return Some(PhysicalCurrentRootAdvanceFailureCause::TransitionIdentityMismatch);
    }
    if current_root.generation().checked_add(1) != Some(durable.current_root_generation()) {
        return Some(PhysicalCurrentRootAdvanceFailureCause::CandidateGenerationMismatch);
    }
    None
}
