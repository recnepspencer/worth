//! Structural verification of one scheme's committed transition against the
//! branch's group posture, reading only the rows the transition names.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::aggregate_identity::transition::group::model::AggregateGroupOrigin;

use super::branch_state::{GroupStateDenial, SchemeGroups};
use super::{CommittedGroupTransition, GroupTransitionKind};

/// Complexity: O(T log G) for T named groups and members.
pub(super) fn verify(
    groups: &SchemeGroups,
    committed: &CommittedGroupTransition,
) -> Result<(), GroupStateDenial> {
    let scheme_id = || committed.scheme_id().to_owned();
    let posture = |group, detail| GroupStateDenial::GroupPosture {
        scheme_id: scheme_id(),
        group,
        detail,
    };
    if groups.rule != committed.scheme.rule() {
        return Err(GroupStateDenial::RuleChanged {
            scheme_id: scheme_id(),
        });
    }
    let transition = &committed.transition;
    for retired in &transition.retired {
        if !groups.live.contains_key(&groups.key(*retired))? {
            return Err(posture(*retired, "only a live group retires"));
        }
    }
    let mut next_owner = BTreeMap::new();
    let mut touched = transition.retired.clone();
    for assignment in &transition.assignments {
        let identity = assignment.identity;
        let key = groups.key(identity);
        let (live, retired) = (
            groups.live.contains_key(&key)?,
            groups.retired.contains_key(&key)?,
        );
        let lawful = match assignment.origin {
            AggregateGroupOrigin::Kept => live && !transition.retired.contains(&identity),
            AggregateGroupOrigin::Born => !live && !retired,
            AggregateGroupOrigin::Revived => retired && !live,
        };
        if !lawful {
            return Err(posture(
                identity,
                "origin contradicts the branch's group state",
            ));
        }
        if !touched.insert(identity) {
            return Err(posture(identity, "one group is assigned twice"));
        }
        if assignment.members.is_empty() {
            return Err(GroupStateDenial::EmptyGroup {
                scheme_id: scheme_id(),
                group: identity,
            });
        }
        for (member, _) in assignment.members.iter() {
            if next_owner.insert(*member, identity).is_some() {
                return Err(GroupStateDenial::MemberOverlap {
                    scheme_id: scheme_id(),
                    member: *member,
                });
            }
        }
    }
    for member in next_owner.keys() {
        if let Some(owner) = groups.member_owner.get(&groups.key(*member))?
            && !touched.contains(&*owner)
        {
            return Err(GroupStateDenial::MemberOverlap {
                scheme_id: scheme_id(),
                member: *member,
            });
        }
    }
    // Preflight every departing member's `GROUP_MEMBER_OWNER` row for touched
    // prior live groups (including dissolve-only commits with empty next_owner)
    // so damaged cold group pages refuse before durable append.
    for identity in &touched {
        if let Some(prior) = groups.live.get(&groups.key(*identity))? {
            for (member, _) in prior.members.iter() {
                if !next_owner.contains_key(member) {
                    let _ = groups.member_owner.get(&groups.key(*member))?;
                }
            }
        }
    }
    verify_events(groups, committed)
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
        match assignment.origin {
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
