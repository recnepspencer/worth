//! Which group of each installed scheme holds a member name, at any basis.
//!
//! Derived from the same committed group batches as the group names: a
//! retirement or a carrier change ends a member's tenure, and a carrier
//! declaration starts it, all at the batch's commit.

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;

use super::entry::ordmap_prev;
use super::journal::IngestionJournal;
use super::{
    Carriers, NameEntry, NameState, PersistentNameBranchIndex, PersistentNameCommittedBatch,
    PersistentNameIndexDenial,
};
use crate::lineage::tiered::NamingReadScope;

/// One member's group tenure under one scheme: strictly ascending commits,
/// each holding until the next entry. Backed by `im::OrdMap` so fork
/// copy-on-write is `O(1)` and historical lookups take one `O(log t)` descent.
pub(crate) type MemberTenure = im::OrdMap<CommitId, Option<AggregateIdentity>>;

/// One member's tenures, by installed group scheme (`O(1)` fork COW).
pub(crate) type MemberTenures = im::OrdMap<String, MemberTenure>;

fn tenure_at(tenure: &MemberTenure, basis: CommitId) -> Option<AggregateIdentity> {
    ordmap_prev(tenure, &basis).and_then(|(_, group)| *group)
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
        scope: &NamingReadScope,
    ) -> Option<AggregateIdentity> {
        let tenure = self.member_groups.get_in(member, scope)?.get(scheme_id)?;
        tenure_at(tenure, basis)
    }

    /// Every `(scheme, group)` holding `member` at `basis`, in scheme order.
    ///
    /// Complexity: O(1) expected probe plus O(S log t) over the member's own
    /// S scheme tenures.
    pub(crate) fn groups_of_member<'s>(
        &'s self,
        member: &PersistentNameIdentity,
        basis: CommitId,
        scope: &'s NamingReadScope,
    ) -> Vec<(&'s str, AggregateIdentity)> {
        let Some(tenures) = self.member_groups.get_in(member, scope) else {
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
    pub(crate) fn group_members_at<'s>(
        &'s self,
        group: AggregateIdentity,
        basis: CommitId,
        scope: &'s NamingReadScope,
    ) -> Option<&'s Carriers> {
        let entry = self.entry_in(&PersistentNameKey::Group(group), scope)?;
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
    ) -> Result<(), PersistentNameIndexDenial> {
        let Some(scheme) = self.group_scheme(group)? else {
            return Ok(());
        };
        for member in members {
            self.set_member_group(&scheme, *member, Some(group), None, commit, journal)?;
        }
        Ok(())
    }

    /// Starts `group`'s tenure over each of `members` under its scheme.
    pub(super) fn start_member_tenures<'m>(
        &mut self,
        group: AggregateIdentity,
        members: impl IntoIterator<Item = &'m PersistentNameIdentity>,
        commit: CommitId,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let Some(scheme) = self.group_scheme(group)? else {
            return Ok(());
        };
        for member in members {
            self.set_member_group(&scheme, *member, None, Some(group), commit, journal)?;
        }
        Ok(())
    }

    fn group_scheme(
        &self,
        group: AggregateIdentity,
    ) -> Result<Option<String>, PersistentNameIndexDenial> {
        let entry = self.names.get(&PersistentNameKey::Group(group))?;
        Ok(entry
            .as_deref()
            .and_then(NameEntry::scheme_id)
            .map(str::to_owned))
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
    ) -> Result<(), PersistentNameIndexDenial> {
        let current = self
            .member_groups
            .get(&member)?
            .as_deref()
            .and_then(|tenures| tenures.get(scheme))
            .and_then(|tenure| tenure.get_max())
            .and_then(|(_, group)| *group);
        if only_from.is_some() && current != only_from {
            return Ok(());
        }
        journal.save_tenure(member, scheme, commit, &self.member_groups);
        let tenures = self
            .member_groups
            .get_or_insert_with(member, Default::default)?;
        let tenure = tenures
            .entry(scheme.to_owned())
            .or_insert_with(im::OrdMap::new);
        tenure.insert(commit, next);
        Ok(())
    }

    /// Applies each declared carrier set as a delta onto the name's current
    /// carriers: the new timeline state shares every unchanged member with
    /// its predecessor, and only members that left or joined move tenures.
    ///
    /// Complexity: O(declared) comparison plus O(delta log M) allocation per
    /// changed name; never a copy of a whole carrier set or history.
    pub(super) fn ingest_carriers(
        &mut self,
        batch: &PersistentNameCommittedBatch,
        journal: &mut IngestionJournal,
    ) -> Result<(), PersistentNameIndexDenial> {
        let mut changed = Vec::new();
        for (key, carriers) in &batch.carriers {
            self.declare_carriers(*key, carriers.iter(), batch.commit, journal, &mut changed)?;
        }
        for assignment in batch.group_assignments() {
            let key = PersistentNameKey::Group(assignment.identity);
            let members = assignment.members.iter().map(|(name, _)| name);
            self.declare_carriers(key, members, batch.commit, journal, &mut changed)?;
        }
        // Every departure ends before any arrival starts, so a member moving
        // between two groups of one scheme ends in its new group.
        for (group, left, _) in &changed {
            self.end_member_tenures(*group, left, batch.commit, journal)?;
        }
        for (group, _, joined) in &changed {
            self.start_member_tenures(*group, joined, batch.commit, journal)?;
        }
        Ok(())
    }

    /// Applies one declared carrier set (ascending) as a delta onto `key`'s
    /// current carriers and records a group's left and joined members.
    fn declare_carriers<'m>(
        &mut self,
        key: PersistentNameKey,
        declared: impl Iterator<Item = &'m PersistentNameIdentity>,
        commit: CommitId,
        journal: &mut IngestionJournal,
        changed: &mut Vec<(AggregateIdentity, MemberDelta, MemberDelta)>,
    ) -> Result<(), PersistentNameIndexDenial> {
        let entry = self.names.get(&key)?;
        let Some(live) = entry.as_deref().and_then(NameEntry::current_live) else {
            return Err(PersistentNameIndexDenial::InvalidTransition {
                name: key,
                detail: "carriers declared for a name that is not live",
            });
        };
        let (left, joined) = carrier_delta(&live.carriers, declared);
        if left.is_empty() && joined.is_empty() {
            return Ok(());
        }
        let mut updated = live.clone();
        drop(entry);
        for member in &left {
            updated.carriers.remove(member);
        }
        for member in &joined {
            updated.carriers.insert(*member);
        }
        self.entry_mut(key, commit, journal)?
            .set_state(commit, NameState::Live(updated));
        if let PersistentNameKey::Group(group) = key {
            changed.push((group, left, joined));
        }
        Ok(())
    }
}

/// Member names that left or joined a carrier set in one commit.
pub(super) type MemberDelta = Vec<PersistentNameIdentity>;

/// The members of `prior` that the ascending `declared` drops, and those it
/// adds, by one merge walk over both.
///
/// Complexity: O(|prior| + |declared|) comparisons, O(delta) allocation.
pub(super) fn carrier_delta<'m>(
    prior: &Carriers,
    declared: impl Iterator<Item = &'m PersistentNameIdentity>,
) -> (MemberDelta, MemberDelta) {
    use std::cmp::Ordering;
    let (mut left, mut joined) = (Vec::new(), Vec::new());
    let (mut old, mut new) = (prior.iter().peekable(), declared.peekable());
    loop {
        match (old.peek(), new.peek()) {
            (None, None) => break,
            (Some(member), None) => {
                left.push(**member);
                old.next();
            }
            (None, Some(member)) => {
                joined.push(**member);
                new.next();
            }
            (Some(was), Some(now)) => match was.cmp(now) {
                Ordering::Less => {
                    left.push(**was);
                    old.next();
                }
                Ordering::Greater => {
                    joined.push(**now);
                    new.next();
                }
                Ordering::Equal => {
                    old.next();
                    new.next();
                }
            },
        }
    }
    (left, joined)
}
