//! Structural verification of one scheme's committed transition against the
//! branch's group posture, reading only the rows the transition names: a
//! kept group's delta must remove current members, change every member it
//! upserts and keep the group nonempty; an entering member must leave its
//! current group in the same transition.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::transition::group::delta::{
    AggregateGroupChange, AggregateGroupMemberDelta,
};
use crate::contracts::aggregate_identity::transition::group::model::AggregateGroupOrigin;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;

use super::branch_state::{GroupStateDenial, SchemeGroups};
use super::group_rows::GroupMemberKey;
use super::{CommittedGroupTransition, GroupTransitionKind};

/// Complexity: O(T log N) for T named groups and touched members, plus each
/// retired group's member rows (preflighted before durable append).
pub(super) fn verify(
    groups: &SchemeGroups,
    committed: &CommittedGroupTransition,
) -> Result<(), GroupStateDenial> {
    let check = Check {
        groups,
        scheme_id: committed.scheme_id(),
    };
    if groups.rule != committed.scheme.rule() {
        return Err(GroupStateDenial::RuleChanged {
            scheme_id: check.scheme_id.to_owned(),
        });
    }
    let transition = &committed.transition;
    for retired in &transition.retired {
        check.retiring(*retired)?;
    }
    let mut moves = Moves::default();
    let mut touched = transition.retired.clone();
    for assignment in &transition.assignments {
        let identity = assignment.identity;
        let key = groups.key(identity);
        let header = groups.live.get(&key)?;
        let retired = groups.retired.contains_key(&key)?;
        let lawful = match assignment.origin() {
            AggregateGroupOrigin::Kept => {
                header.is_some() && !transition.retired.contains(&identity)
            }
            AggregateGroupOrigin::Born => header.is_none() && !retired,
            AggregateGroupOrigin::Revived => retired && header.is_none(),
        };
        if !lawful {
            return Err(check.posture(identity, "origin contradicts the branch's group state"));
        }
        if !touched.insert(identity) {
            return Err(check.posture(identity, "one group is assigned twice"));
        }
        match &assignment.change {
            AggregateGroupChange::Kept(delta) => {
                let size = header.map_or(0, |header| header.counts.total());
                check.kept(identity, size, delta, &mut moves)?;
            }
            AggregateGroupChange::Born(members) | AggregateGroupChange::Revived(members) => {
                if members.is_empty() {
                    return Err(check.empty(identity));
                }
                for (name, _) in members.iter() {
                    moves
                        .enter(*name, identity)
                        .map_err(|name| check.overlap(name))?;
                }
            }
        }
    }
    // An entering member must leave its current group in this transition.
    for member in moves.entering.keys() {
        if let Some(owner) = groups.member_owner.get(&groups.key(*member))?
            && !transition.retired.contains(&owner.group)
            && !moves.released.contains(&(owner.group, *member))
        {
            return Err(check.overlap(*member));
        }
    }
    verify_events(groups, committed)
}

/// Members a transition moves: those kept groups release, and those
/// entering a group.
#[derive(Default)]
struct Moves {
    released: BTreeSet<(AggregateIdentity, PersistentNameIdentity)>,
    entering: BTreeMap<PersistentNameIdentity, AggregateIdentity>,
}

impl Moves {
    /// Records `name` entering `group`; refuses a member entering twice.
    fn enter(
        &mut self,
        name: PersistentNameIdentity,
        group: AggregateIdentity,
    ) -> Result<(), PersistentNameIdentity> {
        match self.entering.insert(name, group) {
            Some(_) => Err(name),
            None => Ok(()),
        }
    }
}

/// One scheme's verification reads and typed refusals.
struct Check<'g> {
    groups: &'g SchemeGroups,
    scheme_id: &'g str,
}

impl Check<'_> {
    fn posture(&self, group: AggregateIdentity, detail: &'static str) -> GroupStateDenial {
        GroupStateDenial::GroupPosture {
            scheme_id: self.scheme_id.to_owned(),
            group,
            detail,
        }
    }

    fn overlap(&self, member: PersistentNameIdentity) -> GroupStateDenial {
        GroupStateDenial::MemberOverlap {
            scheme_id: self.scheme_id.to_owned(),
            member,
        }
    }

    fn empty(&self, group: AggregateIdentity) -> GroupStateDenial {
        GroupStateDenial::EmptyGroup {
            scheme_id: self.scheme_id.to_owned(),
            group,
        }
    }

    /// A retiring group must be live; every departing member's rows are
    /// preflighted so damaged cold group pages refuse before durable append.
    fn retiring(&self, group: AggregateIdentity) -> Result<(), GroupStateDenial> {
        let groups = self.groups;
        if !groups.live.contains_key(&groups.key(group))? {
            return Err(self.posture(group, "only a live group retires"));
        }
        for key in groups.member_keys(group)? {
            let _ = groups.member_owner.get(&groups.key(key.name))?;
        }
        Ok(())
    }

    /// A kept group's delta removes current members, changes every current
    /// member it upserts, and leaves the group nonempty.
    ///
    /// Complexity: O(delta * log N).
    fn kept(
        &self,
        identity: AggregateIdentity,
        mut size: u64,
        delta: &AggregateGroupMemberDelta,
        moves: &mut Moves,
    ) -> Result<(), GroupStateDenial> {
        let groups = self.groups;
        for name in delta.removed() {
            match groups.member_owner.get(&groups.key(*name))?.as_deref() {
                Some(owner) if owner.group == identity => {
                    moves.released.insert((identity, *name));
                    size = size.saturating_sub(1);
                }
                _ => return Err(self.posture(identity, "a kept group removes a non-member")),
            }
        }
        for (name, member) in delta.upserted() {
            match groups.member_owner.get(&groups.key(*name))?.as_deref() {
                Some(owner) if owner.group == identity => {
                    let held = GroupMemberKey {
                        group: identity,
                        role: owner.role,
                        name: *name,
                    };
                    let carrier = groups.members.get(&groups.key(held))?;
                    if owner.role == member.role && carrier.as_deref() == Some(&member.carrier) {
                        return Err(
                            self.posture(identity, "a kept delta repeats an unchanged member")
                        );
                    }
                }
                _ => {
                    size += 1;
                    moves
                        .enter(*name, identity)
                        .map_err(|name| self.overlap(name))?;
                }
            }
        }
        if size == 0 {
            return Err(self.empty(identity));
        }
        Ok(())
    }
}

fn verify_events(
    groups: &SchemeGroups,
    committed: &CommittedGroupTransition,
) -> Result<(), GroupStateDenial> {
    let mismatch = |detail| GroupStateDenial::EventMismatch {
        scheme_id: committed.scheme_id().to_owned(),
        detail,
    };
    let transition = &committed.transition;
    let assigned: BTreeSet<_> = transition.assignments.iter().map(|a| a.identity).collect();
    let mut implied = BTreeSet::new();
    for assignment in &transition.assignments {
        match assignment.origin() {
            AggregateGroupOrigin::Born => implied.insert(Event::Born {
                group: assignment.identity,
            }),
            AggregateGroupOrigin::Revived => implied.insert(Event::Revived {
                group: assignment.identity,
            }),
            AggregateGroupOrigin::Kept => false,
        };
    }
    implied.extend(
        transition
            .retired
            .iter()
            .map(|group| Event::Retired { group: *group }),
    );
    for event in &transition.lineage {
        match *event {
            Event::SplitFrom { fragment, source } => {
                let lawful = committed.kind == GroupTransitionKind::Partition
                    && assigned.contains(&fragment)
                    && fragment != source
                    && groups.live.contains_key(&groups.key(source))?;
                if !lawful {
                    return Err(mismatch(
                        "a split edge must leave a live group for a next-state group",
                    ));
                }
            }
            Event::MergedInto { absorbed, keeper } => {
                let lawful = committed.kind == GroupTransitionKind::Partition
                    && transition.retired.contains(&absorbed)
                    && assigned.contains(&keeper);
                if !lawful {
                    return Err(mismatch(
                        "a merge edge must leave a retired group for a next-state group",
                    ));
                }
            }
            Event::Born { .. } | Event::Revived { .. } | Event::Retired { .. } => {
                if !implied.remove(event) {
                    return Err(mismatch("a lifecycle event the outcome does not imply"));
                }
            }
        }
    }
    if !implied.is_empty() {
        return Err(mismatch(
            "the outcome implies a lifecycle event the commit omits",
        ));
    }
    Ok(())
}
