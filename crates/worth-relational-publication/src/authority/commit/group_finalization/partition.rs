//! Declared group partitions, and member retirements, named by the group
//! naming authority against the branch's prior groups.
//!
//! A scheme's changed scope is every prior live group a claim dissolves,
//! shares a member with a declared group, or loses a member this commit
//! retires. Each scoped group that is not dissolved keeps its undeclared live
//! members together; only `dissolved` lets them leave the scheme. The
//! authority then names the scope's next partition by the scheme's declared
//! continuity rule, so keepers never depend on declaration order or native
//! allocation.

use std::collections::{BTreeMap, BTreeSet};

use contracts::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial as Denial;
use contracts::aggregate_identity::transition::group::model::{
    AggregateGroupMembers, AggregateGroupOrigin, AggregateGroupScheme, AggregatePriorGroup,
};
use contracts::graph_member::NativeMemberKey;
use contracts::lineage::data::naming::PersistentNameIdentity;
use contracts::lineage::data::postures::GroupPartitionClaim;
use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::groups::{
    CommittedGroupTransition, GroupStateRead, GroupTransitionKind, SchemeGroupsRead,
};
use worth_relational_mutation::internal_port::aggregate_identity::AggregateGroupNamingAuthority;

use super::commit_names::CommitNameOutcome;
use super::GroupRefusal;

/// One scheme's merged claims.
struct SchemeScope {
    scheme: AggregateGroupScheme,
    dissolved: BTreeSet<AggregateIdentity>,
    declared: Vec<AggregateGroupMembers>,
}

/// Names every claimed or member-retiring scheme's next partition.
///
/// Complexity: O(S * (M log M)) for S touched schemes of M scoped members.
pub(super) fn partition_transitions(
    claims: &[GroupPartitionClaim],
    state: Option<GroupStateRead<'_>>,
    outcome: &CommitNameOutcome<'_, '_>,
) -> Result<Vec<CommittedGroupTransition>, GroupRefusal> {
    let mut scopes: BTreeMap<String, SchemeScope> = BTreeMap::new();
    for claim in claims {
        let scheme_id = claim.scheme.id().as_str().to_owned();
        let refuse = |denial| GroupRefusal::new(&scheme_id, denial);
        let scope = scopes
            .entry(scheme_id.clone())
            .or_insert_with(|| SchemeScope {
                scheme: claim.scheme.clone(),
                dissolved: BTreeSet::new(),
                declared: Vec::new(),
            });
        if scope.scheme != claim.scheme {
            return Err(refuse(Denial::ContinuityRuleChanged));
        }
        scope.dissolved.extend(claim.dissolved.iter().copied());
        for group in &claim.groups {
            let mut members = AggregateGroupMembers::new();
            for (name, role) in group {
                let entity = outcome
                    .member_after(name)
                    .ok_or_else(|| refuse(Denial::MemberNotLive { name: *name }))?;
                members
                    .insert(*name, NativeMemberKey::Entity(entity), *role)
                    .map_err(refuse)?;
            }
            scope.declared.push(members);
        }
    }
    let retired = outcome.retired_names();
    for (scheme_id, groups) in state.iter().flat_map(|state| state.schemes()) {
        if scopes.contains_key(scheme_id) {
            continue;
        }
        if retired.iter().any(|name| groups.owner_of(name).is_some()) {
            let scheme = scheme_of(scheme_id, groups.rule())
                .map_err(|denial| GroupRefusal::new(scheme_id, denial))?;
            let scope = SchemeScope {
                scheme,
                dissolved: BTreeSet::new(),
                declared: Vec::new(),
            };
            scopes.insert(scheme_id.to_owned(), scope);
        }
    }
    let mut transitions = Vec::new();
    for (scheme_id, scope) in scopes {
        let groups = state.and_then(|state| state.scheme(&scheme_id));
        let refuse = |denial| GroupRefusal::new(&scheme_id, denial);
        let committed = name_scope(scope, groups, &retired, outcome).map_err(refuse)?;
        if let Some(committed) = committed {
            transitions.push(committed);
        }
    }
    Ok(transitions)
}

fn scheme_of(
    scheme_id: &str,
    rule: contracts::aggregate_identity::transition::group::model::AggregateContinuityRule,
) -> Result<AggregateGroupScheme, Denial> {
    let id = contracts::aggregate_identity::transition::group::model::AggregateGroupSchemeId::new(
        scheme_id,
    )?;
    Ok(AggregateGroupScheme::new(id, rule))
}

fn name_scope(
    scope: SchemeScope,
    groups: Option<SchemeGroupsRead<'_>>,
    retired: &[PersistentNameIdentity],
    outcome: &CommitNameOutcome<'_, '_>,
) -> Result<Option<CommittedGroupTransition>, Denial> {
    let SchemeScope {
        scheme,
        dissolved,
        mut declared,
    } = scope;
    if groups.is_some_and(|groups| groups.rule() != scheme.rule()) {
        return Err(Denial::ContinuityRuleChanged);
    }
    let live = |identity: &AggregateIdentity| groups.and_then(|groups| groups.group(identity));
    for identity in &dissolved {
        if live(identity).is_none() {
            return Err(Denial::UnknownGroup {
                identity: *identity,
            });
        }
    }
    let declared_names: BTreeSet<PersistentNameIdentity> = declared
        .iter()
        .flat_map(|members| members.iter().map(|(name, _)| *name))
        .collect();
    let owner = |name: &PersistentNameIdentity| groups.and_then(|groups| groups.owner_of(name));
    let mut scoped: BTreeSet<AggregateIdentity> = dissolved.clone();
    scoped.extend(declared_names.iter().filter_map(owner));
    scoped.extend(retired.iter().filter_map(owner));
    let mut prior: Vec<AggregatePriorGroup> = Vec::with_capacity(scoped.len());
    for identity in &scoped {
        let group = live(identity)
            .expect("scoped groups are live owners")
            .clone();
        if !dissolved.contains(identity) {
            let mut residual = AggregateGroupMembers::new();
            for (name, member) in group.members.iter() {
                if declared_names.contains(name) {
                    continue;
                }
                if let Some(entity) = outcome.member_after(name) {
                    residual.insert(*name, NativeMemberKey::Entity(entity), member.role)?;
                }
            }
            if !residual.is_empty() {
                declared.push(residual);
            }
        }
        prior.push(group);
    }
    let transition = AggregateGroupNamingAuthority::transition_with_retired(
        &scheme,
        &prior,
        |identity| groups.is_some_and(|groups| groups.is_retired(identity)),
        declared,
    )?;
    let noop = transition.retired.is_empty()
        && transition.lineage.is_empty()
        && transition.assignments.iter().all(|assignment| {
            assignment.origin == AggregateGroupOrigin::Kept
                && live(&assignment.identity)
                    .is_some_and(|prior| prior.members == assignment.members)
        });
    Ok((!noop).then_some(CommittedGroupTransition {
        kind: GroupTransitionKind::Partition,
        scheme,
        transition,
    }))
}
