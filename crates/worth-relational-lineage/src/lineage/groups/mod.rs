//! Authoritative per-branch group naming state.
//!
//! A commit's group lineage is one [`CommittedGroupTransition`] per touched
//! installed group scheme, carried canonically in the commit's published
//! lineage artifact. Every branch's live groups, retired group names and
//! member ownership are derived by applying those transitions in publication
//! order along the branch's fork ancestry, so fork, checkpoint restore, cold
//! open and tail replay all rebuild the same state from canonical history.

pub mod branch_state;
pub mod committed;
mod group_diff;
mod group_install;
pub mod group_read;
mod group_rows;
mod group_verify;
#[cfg(test)]
mod kept_delta_conformance_trace;
#[cfg(test)]
mod kept_delta_installation;

pub use branch_state::{BranchGroupState, GroupStateAt, GroupStateDenial, SchemeGroups};
pub use committed::{CommittedGroupTransition, GroupTransitionKind};
pub use group_read::{GroupStateRead, SchemeGroupsRead};
