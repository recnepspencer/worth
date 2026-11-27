//! Incremental group naming: the same names, keepers, lineage and refusals as
//! [`super::AggregateGroupNamingAuthority`] over the complete partition, read
//! from a branch's live group state through `O(log M)` point and ordered
//! probes, so a `k`-member edit of an `M`-member group costs
//! `O((k + scoped groups) log M)` and records the kept group's member delta.
//!
//! The scope is what a commit touches: declared groups, members that leave
//! their scheme, members whose carrier changes, and dissolved groups. Every
//! other member of a scoped group stays together in its group's *residual*,
//! which is never materialised unless it is born under a new name.
//!
//! Preconditions the branch group state guarantees: every member belongs to
//! at most one live group, and carriers are an injective function of member
//! names (a live name has one holder), so two proposed members never share a
//! carrier unless both are declared. The full-partition authority stays the
//! differential oracle for exactly this input domain.
use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use super::{AggregateGroupMembers, AggregateGroupTransition, Name};
use super::{AggregateGroupNamingDenial as Denial, AggregateGroupScheme, AggregateGroupStateView};
use crate::contracts::graph_member::NativeMemberKey;

mod assemble;
mod naming;
mod scope;

/// What one commit touches in one installed group scheme.
#[derive(Debug, Default)]
pub struct AggregateGroupScope {
    /// Prior groups whose undeclared members leave the scheme.
    pub dissolved: BTreeSet<AggregateIdentity>,
    /// Groups declared with their complete next membership.
    pub declared: Vec<AggregateGroupMembers>,
    /// Members leaving their group and the scheme (retired, or no longer an
    /// entity's live name).
    pub departed: BTreeSet<Name>,
    /// Members staying in their group under a changed carrier.
    pub carriers: BTreeMap<Name, NativeMemberKey>,
}

/// Relational authority for incremental, allocation-independent group names.
pub struct AggregateGroupDeltaAuthority;

impl AggregateGroupDeltaAuthority {
    /// Names `scope`'s next partition against the live groups `state` holds.
    ///
    /// Complexity: O((T + S + P) log M) for T touched names, S scoped prior
    /// groups and P proposed groups, plus O(M) only to materialise a born or
    /// revived group's members or to diff a kept group against a dissolved
    /// or differently scoped prior. Fail-closed: any refusal returns before
    /// a single assignment, retirement or event exists.
    pub fn transition(
        scheme: &AggregateGroupScheme,
        state: &impl AggregateGroupStateView,
        scope: AggregateGroupScope,
    ) -> Result<AggregateGroupTransition, Denial> {
        let proposal = scope::Proposal::resolve(scheme, state, scope)?;
        let plan = naming::Plan::name(scheme, proposal)?;
        plan.guard_live_collisions()?;
        plan.assemble()
    }
}
