//! Group continuity of a history selection (`undo_step`, `redo_to`,
//! `checkout`).
//!
//! A selection rematerializes its model's records exactly as they were at the
//! selected commit, so the model's groups return exactly as they were there
//! too: every scoped current group the selected state does not hold retires,
//! and every scoped selected group comes back under its exact recorded
//! identity, anchor, sources and members (`Revived` when this branch retired
//! it, `Born` when this branch never held it). A selection names no new group
//! and records no split or merge edge.

use std::collections::{BTreeMap, BTreeSet};

use contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as Event;
use contracts::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial as Denial;
use contracts::aggregate_identity::transition::group::delta::{
    AggregateGroupAssignment, AggregateGroupChange,
};
use contracts::aggregate_identity::transition::group::model::{
    AggregateGroupMembers, AggregateGroupScheme, AggregateGroupSchemeId, AggregatePriorGroup,
};
use contracts::aggregate_identity::transition::group::transition_result::{
    member_delta, AggregateGroupTransition,
};
use contracts::graph_member::NativeMemberKey;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::record_ids::EntityId;
use worth_foundational::facade::AggregateIdentity;
use contracts::lineage::data::naming::PersistentNameHolder;
use contracts::lineage::data::postures::LineageBasisView;
use worth_foundational::FoundationalBranchTarget;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::{
    CanonicalOwnedGraphSelection, CanonicalOwnedGraphSelectionOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::groups::{
    BranchGroupState, CommittedGroupTransition, GroupStateAt, GroupStateRead, GroupTransitionKind,
    SchemeGroupsRead,
};
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::naming_basis_binding::BranchNamingBasisGuard;

use super::commit_names::CommitNameOutcome;
use super::GroupRefusal;

/// Restores the selection model's groups to the selected commit's state.
///
/// Complexity: O(G log G) over the live groups of both states, plus the
/// selected state: borrowed in O(1) at the head, else rebuilt from its
/// published group lineage through the selected commit.
pub(super) fn selection_transitions(
    selection: &CanonicalOwnedGraphSelection,
    basis: &BranchNamingBasisGuard<'_>,
    current: Option<GroupStateRead<'_>>,
    outcome: &CommitNameOutcome<'_, '_>,
) -> Result<Vec<CommittedGroupTransition>, GroupRefusal> {
    let selected = selection.selected_basis();
    let target = match selected.reference().target() {
        FoundationalBranchTarget::Empty => GroupStateAt::Rebuilt(BranchGroupState::default()),
        FoundationalBranchTarget::Basis(target) => basis
            .group_state_at(selected.branch_id(), CommitId(target.selected_commit_id()))
            .map_err(GroupRefusal::unavailable)?,
    };
    let target = target.read(basis.scope());
    let current_entities: BTreeSet<EntityId> =
        selection.current_entities().iter().copied().collect();
    let selected_entities: BTreeSet<EntityId> = selection
        .selected_entities()
        .map(|(entity, _)| entity)
        .collect();
    let mut schemes: BTreeMap<&str, (Option<SchemeGroupsRead<'_>>, Option<SchemeGroupsRead<'_>>)> =
        BTreeMap::new();
    for (scheme_id, groups) in current.iter().flat_map(|state| state.schemes()) {
        schemes.entry(scheme_id).or_default().0 = Some(groups);
    }
    for (scheme_id, groups) in target.schemes() {
        schemes.entry(scheme_id).or_default().1 = Some(groups);
    }
    let mut transitions = Vec::new();
    for (scheme_id, (now, then)) in schemes {
        let refuse = |denial| GroupRefusal::new(scheme_id, denial);
        let rule = now.or(then).expect("one side holds the scheme").rule();
        if then.is_some_and(|then| then.rule() != rule) {
            return Err(refuse(Denial::ContinuityRuleChanged));
        }
        let in_current = |group: &AggregatePriorGroup| {
            group.members.iter().any(|(name, _)| {
                matches!(basis.live_holder(&name.carrier_name()), Some(PersistentNameHolder::Entity(entity))
                    if current_entities.contains(entity))
            })
        };
        let in_selected = |group: &AggregatePriorGroup| {
            group
                .members
                .iter()
                .any(|(_, member)| selected_entities.contains(&member.carrier.carrier()))
        };
        let now_live: Vec<AggregatePriorGroup> = now.iter().flat_map(|now| now.live()).collect();
        let then_live: Vec<AggregatePriorGroup> =
            then.iter().flat_map(|then| then.live()).collect();
        let leaving: Vec<_> = now_live.iter().filter(|g| in_current(g)).collect();
        let arriving: Vec<_> = then_live.iter().filter(|g| in_selected(g)).collect();
        let live: BTreeMap<_, _> = now_live.iter().map(|g| (g.identity, g)).collect();
        let transition = restore(now, &live, &leaving, &arriving, outcome).map_err(refuse)?;
        if transition.assignments.is_empty() && transition.retired.is_empty() {
            continue;
        }
        let id = AggregateGroupSchemeId::new(scheme_id).map_err(refuse)?;
        transitions.push(CommittedGroupTransition {
            kind: GroupTransitionKind::Selection,
            scheme: AggregateGroupScheme::new(id, rule),
            transition,
        });
    }
    Ok(transitions)
}

fn restore(
    now: Option<SchemeGroupsRead<'_>>,
    live: &BTreeMap<AggregateIdentity, &AggregatePriorGroup>,
    leaving: &[&AggregatePriorGroup],
    arriving: &[&AggregatePriorGroup],
    outcome: &CommitNameOutcome<'_, '_>,
) -> Result<AggregateGroupTransition, Denial> {
    let mut transition = AggregateGroupTransition {
        assignments: Vec::new(),
        retired: BTreeSet::new(),
        lineage: BTreeSet::new(),
    };
    let arriving_ids: BTreeSet<_> = arriving.iter().map(|group| group.identity).collect();
    for group in leaving {
        if !arriving_ids.contains(&group.identity) {
            transition.retired.insert(group.identity);
            transition.lineage.insert(Event::Retired {
                group: group.identity,
            });
        }
    }
    for group in arriving {
        let mut members = AggregateGroupMembers::new();
        for (name, member) in group.members.iter() {
            let carrier_name = name.carrier_name();
            let entity = outcome
                .member_after(&carrier_name)
                .ok_or(Denial::MemberNotLive { name: carrier_name })?;
            let carrier = match &member.carrier {
                NativeMemberKey::Entity(_) => NativeMemberKey::Entity(entity),
                NativeMemberKey::Projected(projected) => {
                    let coordinate = contracts::graph_member::key::ProjectedMemberCoordinate {
                        domain: projected.domain.clone(),
                        carrier: entity,
                        case: projected.case.clone(),
                    };
                    NativeMemberKey::Projected(std::sync::Arc::new(coordinate).into())
                }
            };
            members.insert(name.clone(), carrier, member.role)?;
        }
        let change = match live.get(&group.identity) {
            Some(live) if live.members == members => continue,
            Some(live) => AggregateGroupChange::Kept(member_delta(&live.members, &members)?),
            None if now.is_some_and(|now| now.is_retired(&group.identity)) => {
                transition.lineage.insert(Event::Revived {
                    group: group.identity,
                });
                AggregateGroupChange::Revived(members)
            }
            None => {
                transition.lineage.insert(Event::Born {
                    group: group.identity,
                });
                AggregateGroupChange::Born(members)
            }
        };
        transition.assignments.push(AggregateGroupAssignment {
            identity: group.identity,
            anchor: group.anchor.clone(),
            sources: group.sources.clone(),
            change,
        });
    }
    transition
        .assignments
        .sort_by_key(|assignment| assignment.identity);
    Ok(transition)
}
