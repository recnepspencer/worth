//! Which group of each installed scheme holds a member name, at any basis.
//!
//! Derived from the same committed group batches as the group names: a
//! retirement or a carrier change ends a member's tenure, and a carrier
//! declaration starts it, all at the batch's commit.

use std::collections::BTreeMap;

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;

use super::journal::IngestionJournal;
use super::{Carriers, NameEntry, NameState, PersistentNameBranchIndex};

/// One member's group tenure under one scheme: strictly ascending commits,
/// each holding until the next entry.
pub(crate) type MemberTenure = Vec<(CommitId, Option<AggregateIdentity>)>;

/// One member's tenures, by installed group scheme.
pub(crate) type MemberTenures = BTreeMap<String, MemberTenure>;

fn tenure_at(tenure: &MemberTenure, basis: CommitId) -> Option<AggregateIdentity> {
    let after = tenure.partition_point(|(commit, _)| *commit <= basis);
    tenure.get(after.checked_sub(1)?)?.1
}

impl PersistentNameBranchIndex {
    /// The live group of `scheme_id` holding `member` at `basis`.
    ///
    /// Complexity: O(1) expected probe plus O(log t) in that member's own
    /// tenure under the scheme.
    pub(crate) fn group_of_member(
        &self,
        scheme_id: &str,
        member: &PersistentNameIdentity,
        basis: CommitId,
    ) -> Option<AggregateIdentity> {
        let tenure = self.member_groups.get(member)?.get(scheme_id)?;
        tenure_at(tenure, basis)
    }

    /// Every `(scheme, group)` holding `member` at `basis`, in scheme order.
    ///
    /// Complexity: O(1) expected probe plus O(S log t) over the member's own
    /// S scheme tenures.
    pub(crate) fn groups_of_member(
        &self,
        member: &PersistentNameIdentity,
        basis: CommitId,
    ) -> Vec<(&str, AggregateIdentity)> {
        let Some(tenures) = self.member_groups.get(member) else {
            return Vec::new();
        };
        tenures
            .iter()
            .filter_map(|(scheme, tenure)| Some((scheme.as_str(), tenure_at(tenure, basis)?)))
            .collect()
    }

    /// The members `group` declared at `basis`, when it was live there.
    ///
    /// Complexity: O(log t) in the group's own timeline.
    pub(crate) fn group_members_at(
        &self,
        group: AggregateIdentity,
        basis: CommitId,
    ) -> Option<&Carriers> {
        let entry = self.names.get(&PersistentNameKey::Group(group))?;
        match entry.state_at(basis)?.1 {
            NameState::Live(live) => Some(&live.carriers),
            NameState::Ended => None,
        }
    }

    /// Ends every current member tenure `group` holds under its scheme.
    ///
    /// Complexity: O(|members| log) — only the members whose tenure moves.
    pub(super) fn end_member_tenures<'m>(
        &mut self,
        group: AggregateIdentity,
        members: impl IntoIterator<Item = &'m PersistentNameIdentity>,
        commit: CommitId,
        journal: &mut IngestionJournal,
    ) {
        let Some(scheme) = self.group_scheme(group) else {
            return;
        };
        for member in members {
            self.set_member_group(&scheme, *member, Some(group), None, commit, journal);
        }
    }

    /// Starts `group`'s tenure over each of `members` under its scheme.
    pub(super) fn start_member_tenures<'m>(
        &mut self,
        group: AggregateIdentity,
        members: impl IntoIterator<Item = &'m PersistentNameIdentity>,
        commit: CommitId,
        journal: &mut IngestionJournal,
    ) {
        let Some(scheme) = self.group_scheme(group) else {
            return;
        };
        for member in members {
            self.set_member_group(&scheme, *member, None, Some(group), commit, journal);
        }
    }

    fn group_scheme(&self, group: AggregateIdentity) -> Option<String> {
        self.names
            .get(&PersistentNameKey::Group(group))
            .and_then(NameEntry::scheme_id)
            .map(str::to_owned)
    }

    /// Moves `member`'s tenure to `next` at `commit`; ending (`only_from`
    /// set) changes nothing unless that group still holds the member.
    fn set_member_group(
        &mut self,
        scheme: &str,
        member: PersistentNameIdentity,
        only_from: Option<AggregateIdentity>,
        next: Option<AggregateIdentity>,
        commit: CommitId,
        journal: &mut IngestionJournal,
    ) {
        let current = self
            .member_groups
            .get(&member)
            .and_then(|tenures| tenures.get(scheme))
            .and_then(|tenure| tenure.last())
            .and_then(|(_, group)| *group);
        if only_from.is_some() && current != only_from {
            return;
        }
        journal.save_tenure(member, scheme, self.member_groups.get(&member));
        let tenures = self.member_groups.entry(member).or_default();
        let tenure = tenures.entry(scheme.to_owned()).or_default();
        match tenure.last_mut() {
            Some((at, group)) if *at == commit => *group = next,
            _ => tenure.push((commit, next)),
        }
    }
}
