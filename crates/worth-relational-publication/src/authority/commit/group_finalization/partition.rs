//! Declared group partitions, member retirements and member re-bindings,
//! named by the incremental group naming authority against the branch's
//! live groups.
//!
//! A scheme's changed scope is every live group a claim dissolves, shares a
//! member with a declared group, loses a member this commit retires, or
//! holds a member this commit re-binds to another record. Each scoped group
//! that is not dissolved keeps its undeclared live members together; only
//! `dissolved` lets them leave the scheme. The authority names the scope's
//! next partition by the scheme's declared continuity rule from the touched
//! members alone, so keepers never depend on declaration order or native
//! allocation, and a kept group's untouched members are never read.

use std::collections::{BTreeMap, BTreeSet};

use contracts::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial as Denial;
use contracts::aggregate_identity::transition::group::model::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateGroupScheme, AggregateGroupSchemeId,
};
use contracts::aggregate_identity::transition::group::state_view::AggregateGroupStateView;
use contracts::graph_member::NativeMemberKey;
use contracts::lineage::data::naming::PersistentNameIdentity;
use contracts::lineage::data::postures::GroupPartitionClaim;
use worth_foundational::facade::AggregateIdentity;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::groups::{
    CommittedGroupTransition, GroupStateRead, GroupTransitionKind, SchemeGroupsRead,
};
use worth_relational_mutation::internal_port::aggregate_identity::{
    AggregateGroupDeltaAuthority, AggregateGroupScope,
};

use super::commit_names::CommitNameOutcome;
use super::empty_state::EmptyGroupState;
use super::GroupRefusal;

/// One scheme's merged claims.
struct SchemeScope {
    scheme: AggregateGroupScheme,
    dissolved: BTreeSet<AggregateIdentity>,
    declared: Vec<AggregateGroupMembers>,
}

/// Names every claimed, member-retiring or member-re-binding scheme's next
/// partition.
///
/// Complexity: O(S * (T + scoped groups) * log M) for S touched schemes and
/// T touched members, independent of untouched members.
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
        let retires = retired.iter().any(|name| groups.owner_of(name).is_some());
        let rebinds = outcome
            .bound_names()
            .any(|(name, entity)| rebinding(groups, name, *entity).is_some());
        if retires || rebinds {
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

/// How this commit re-binds group member `name`: `Some(Some(carrier))` for
/// a new entity carrier, `Some(None)` when it no longer names an entity,
/// `None` when `name` is no member or keeps its carrier.
fn rebinding(
    groups: SchemeGroupsRead<'_>,
    name: &PersistentNameIdentity,
    entity: Option<contracts::identity::data::record_ids::EntityId>,
) -> Option<Option<NativeMemberKey>> {
    let membership = groups.membership(name)?;
    match entity.map(NativeMemberKey::Entity) {
        Some(carrier) if carrier == membership.member.carrier => None,
        carrier => Some(carrier),
    }
}

fn scheme_of(
    scheme_id: &str,
    rule: AggregateContinuityRule,
) -> Result<AggregateGroupScheme, Denial> {
    Ok(AggregateGroupScheme::new(
        AggregateGroupSchemeId::new(scheme_id)?,
        rule,
    ))
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
        declared,
    } = scope;
    if groups.is_some_and(|groups| groups.rule() != scheme.rule()) {
        return Err(Denial::ContinuityRuleChanged);
    }
    for identity in &dissolved {
        if groups.and_then(|groups| groups.header(identity)).is_none() {
            return Err(Denial::UnknownGroup {
                identity: *identity,
            });
        }
    }
    let declared_names: BTreeSet<PersistentNameIdentity> = declared
        .iter()
        .flat_map(|members| members.iter().map(|(name, _)| *name))
        .collect();
    let mut scope = AggregateGroupScope {
        dissolved,
        declared,
        ..AggregateGroupScope::default()
    };
    if let Some(groups) = groups {
        let owned = |name: &PersistentNameIdentity| groups.owner_of(name).is_some();
        scope
            .departed
            .extend(retired.iter().filter(|name| owned(name)).copied());
        for (name, entity) in outcome.bound_names() {
            if declared_names.contains(name) {
                continue;
            }
            match rebinding(groups, name, *entity) {
                Some(Some(carrier)) => {
                    scope.carriers.insert(*name, carrier);
                }
                Some(None) => {
                    scope.departed.insert(*name);
                }
                None => {}
            }
        }
    }
    let transition = match groups {
        Some(groups) => AggregateGroupDeltaAuthority::transition(&scheme, &groups, scope)?,
        None => AggregateGroupDeltaAuthority::transition(&scheme, &EmptyGroupState, scope)?,
    };
    Ok((!transition.is_noop()).then_some(CommittedGroupTransition {
        kind: GroupTransitionKind::Partition,
        scheme,
        transition,
    }))
}
