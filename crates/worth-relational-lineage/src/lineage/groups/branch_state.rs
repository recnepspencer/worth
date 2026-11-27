//! One branch's live groups per installed scheme, derived by applying each
//! commit's [`CommittedGroupTransition`]s in publication order.
//!
//! Application is structural verification, never naming: it refuses any
//! transition that retires, keeps or revives a group the branch does not hold
//! in that posture, overlaps two live groups' members, or carries lineage
//! events its own outcome does not imply. Keeper choice and identity minting
//! belong to the group naming authority, which recovery re-runs separately.

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use crate::contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupOrigin, AggregatePriorGroup,
};
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;

use super::{CommittedGroupTransition, GroupTransitionKind};

/// Why a committed group transition cannot apply to a branch's group state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupStateDenial {
    /// One commit carries two transitions for one scheme.
    DuplicateScheme { scheme_id: String },
    /// The transition's continuity rule differs from the scheme's history.
    RuleChanged { scheme_id: String },
    /// The named group is not in the posture the transition requires.
    GroupPosture {
        scheme_id: String,
        group: AggregateIdentity,
        detail: &'static str,
    },
    /// A next-state group has no members.
    EmptyGroup {
        scheme_id: String,
        group: AggregateIdentity,
    },
    /// One member would belong to two live groups of the scheme.
    MemberOverlap {
        scheme_id: String,
        member: PersistentNameIdentity,
    },
    /// The lineage events disagree with the transition's own outcome.
    EventMismatch {
        scheme_id: String,
        detail: &'static str,
    },
}

crate::lineage::clone_fences::assert_not_clone!(SchemeGroups);
crate::lineage::clone_fences::assert_not_clone!(BranchGroupState);

/// One installed scheme's live groups, retired names and member owners.
///
/// Not `Clone`: it holds every live group's member set. A fork copies it
/// only through [`BranchGroupState::duplicate_for_branch_fork`].
#[derive(Debug, Eq, PartialEq)]
pub struct SchemeGroups {
    rule: AggregateContinuityRule,
    live: BTreeMap<AggregateIdentity, AggregatePriorGroup>,
    retired: BTreeSet<AggregateIdentity>,
    member_owner: BTreeMap<PersistentNameIdentity, AggregateIdentity>,
}

impl SchemeGroups {
    fn new(rule: AggregateContinuityRule) -> Self {
        Self {
            rule,
            live: BTreeMap::new(),
            retired: BTreeSet::new(),
            member_owner: BTreeMap::new(),
        }
    }

    pub const fn rule(&self) -> AggregateContinuityRule {
        self.rule
    }

    /// Live groups in identity order.
    pub fn live(&self) -> impl Iterator<Item = &AggregatePriorGroup> {
        self.live.values()
    }

    pub fn group(&self, identity: &AggregateIdentity) -> Option<&AggregatePriorGroup> {
        self.live.get(identity)
    }

    /// Every group name this scheme has retired and not revived.
    pub fn retired(&self) -> &BTreeSet<AggregateIdentity> {
        &self.retired
    }

    /// The live group `member` belongs to.
    ///
    /// Complexity: O(log M) in the scheme's live member count.
    pub fn owner_of(&self, member: &PersistentNameIdentity) -> Option<AggregateIdentity> {
        self.member_owner.get(member).copied()
    }
}

/// Every installed scheme's groups on one branch.
///
/// Not `Clone`: a copy is O(groups × members). Head reads borrow the
/// maintained state; a branch fork copies it through
/// [`Self::duplicate_for_branch_fork`].
#[derive(Debug, Default, Eq, PartialEq)]
pub struct BranchGroupState {
    schemes: BTreeMap<String, SchemeGroups>,
    latest_commit: Option<CommitId>,
}

impl BranchGroupState {
    /// A forked branch's explicit O(groups × members) copy of this state.
    pub fn duplicate_for_branch_fork(&self) -> Self {
        let schemes = self.schemes.iter().map(|(id, groups)| {
            let copy = SchemeGroups {
                rule: groups.rule,
                live: groups.live.clone(),
                retired: groups.retired.clone(),
                member_owner: groups.member_owner.clone(),
            };
            (id.clone(), copy)
        });
        Self {
            schemes: schemes.collect(),
            latest_commit: self.latest_commit,
        }
    }

    pub fn scheme(&self, scheme_id: &str) -> Option<&SchemeGroups> {
        self.schemes.get(scheme_id)
    }

    pub fn schemes(&self) -> impl Iterator<Item = (&str, &SchemeGroups)> {
        self.schemes
            .iter()
            .map(|(id, groups)| (id.as_str(), groups))
    }

    /// The newest commit whose transitions this state applied.
    pub const fn latest_commit(&self) -> Option<CommitId> {
        self.latest_commit
    }

    /// Verifies one commit's transitions against this state without applying
    /// them.
    ///
    /// Complexity: as [`Self::apply`].
    pub fn verify(&self, transitions: &[CommittedGroupTransition]) -> Result<(), GroupStateDenial> {
        let mut seen = BTreeSet::new();
        for committed in transitions {
            let scheme_id = committed.scheme_id();
            if !seen.insert(scheme_id) {
                return Err(GroupStateDenial::DuplicateScheme {
                    scheme_id: scheme_id.to_owned(),
                });
            }
            let empty = SchemeGroups::new(committed.scheme.rule());
            verify(self.schemes.get(scheme_id).unwrap_or(&empty), committed)?;
        }
        Ok(())
    }

    /// Applies one commit's transitions atomically: every transition is
    /// verified against the state before any of them changes it.
    ///
    /// Complexity: O(T log G) for T touched groups and members and G live
    /// groups and members of the touched schemes.
    pub fn apply(
        &mut self,
        commit: CommitId,
        transitions: &[CommittedGroupTransition],
    ) -> Result<(), GroupStateDenial> {
        self.verify(transitions)?;
        for committed in transitions {
            self.schemes
                .entry(committed.scheme_id().to_owned())
                .or_insert_with(|| SchemeGroups::new(committed.scheme.rule()))
                .install(committed);
        }
        if !transitions.is_empty() {
            self.latest_commit = Some(self.latest_commit.map_or(commit, |at| at.max(commit)));
        }
        Ok(())
    }
}

/// A branch's group state as of one commit: the maintained state itself
/// when no later group lineage applies to it, else one rebuilt for that
/// historical commit.
#[derive(Debug)]
pub enum GroupStateAt<'state> {
    Head(&'state BranchGroupState),
    Rebuilt(BranchGroupState),
}

impl std::ops::Deref for GroupStateAt<'_> {
    type Target = BranchGroupState;

    fn deref(&self) -> &BranchGroupState {
        match self {
            Self::Head(state) => state,
            Self::Rebuilt(state) => state,
        }
    }
}

fn verify(
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
        if !groups.live.contains_key(retired) {
            return Err(posture(*retired, "only a live group retires"));
        }
    }
    let mut next_owner = BTreeMap::new();
    let mut touched: BTreeSet<AggregateIdentity> = transition.retired.clone();
    for assignment in &transition.assignments {
        let identity = assignment.identity;
        let (live, retired) = (
            groups.live.contains_key(&identity),
            groups.retired.contains(&identity),
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
        if let Some(owner) = groups.member_owner.get(member)
            && !touched.contains(owner)
        {
            return Err(GroupStateDenial::MemberOverlap {
                scheme_id: scheme_id(),
                member: *member,
            });
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
                    && groups.live.contains_key(&source);
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

impl SchemeGroups {
    fn install(&mut self, committed: &CommittedGroupTransition) {
        let transition = &committed.transition;
        let touched: BTreeSet<_> = transition
            .retired
            .iter()
            .copied()
            .chain(transition.assignments.iter().map(|a| a.identity))
            .collect();
        for identity in &touched {
            if let Some(prior) = self.live.remove(identity) {
                for (member, _) in prior.members.iter() {
                    if self.member_owner.get(member) == Some(identity) {
                        self.member_owner.remove(member);
                    }
                }
            }
        }
        self.retired.extend(transition.retired.iter().copied());
        for assignment in &transition.assignments {
            self.retired.remove(&assignment.identity);
            for (member, _) in assignment.members.iter() {
                self.member_owner.insert(*member, assignment.identity);
            }
            self.live.insert(
                assignment.identity,
                AggregatePriorGroup {
                    identity: assignment.identity,
                    anchor: assignment.anchor,
                    sources: assignment.sources.clone(),
                    members: assignment.members.clone(),
                },
            );
        }
    }
}
