//! Admit complete touched graph partitions against the commit's naming basis.
use contracts::aggregate_identity::transition::group::denial::AggregateGroupNamingDenial as Denial;
use contracts::aggregate_identity::transition::group::model::{
    AggregateGroupMembers, AggregateMemberRole, CanonicalGroupMemberIdentity,
};
use contracts::aggregate_identity::transition::group::state_view::AggregateGroupStateView as _;
use contracts::graph_member::NativeMemberKey;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::groups::{
    CommittedGroupTransition, GroupStateRead, GroupTransitionKind,
};
use worth_relational_mutation::internal_port::aggregate_identity::{
    cohorts::TouchedAggregatePartitionFact, AggregateGroupDeltaAuthority, AggregateGroupScope,
};

use super::{commit_names::CommitNameOutcome, empty_state::EmptyGroupState, GroupRefusal};

pub(super) fn transitions(
    facts: &[TouchedAggregatePartitionFact],
    state: Option<GroupStateRead<'_>>,
    outcome: &CommitNameOutcome<'_, '_>,
) -> Result<Vec<CommittedGroupTransition>, GroupRefusal> {
    let mut result = Vec::new();
    for fact in facts {
        let scheme = fact.scheme();
        let groups = state.and_then(|state| state.scheme(scheme.id().as_str()));
        let refuse = |denial| GroupRefusal::new(scheme.id().as_str(), denial);
        let mut scope = AggregateGroupScope::default();
        for prior in fact.prior() {
            for member in prior {
                let key =
                    member_key(member, |carrier| outcome.name_before(carrier)).map_err(refuse)?;
                let held = groups
                    .and_then(|groups| groups.membership(&key))
                    .filter(|held| held.member.carrier == *member)
                    .ok_or_else(|| refuse(Denial::MissingPriorMember { name: key }))?;
                scope.dissolved.insert(held.group);
            }
        }
        for proposed in fact.proposed() {
            let mut members = AggregateGroupMembers::new();
            for member in proposed {
                let key =
                    member_key(member, |carrier| outcome.name_after(carrier)).map_err(refuse)?;
                members
                    .insert(key, member.clone(), AggregateMemberRole::Ordinary)
                    .map_err(refuse)?;
            }
            scope.declared.push(members);
        }
        let transition = match groups {
            Some(groups) => AggregateGroupDeltaAuthority::transition(scheme, &groups, scope),
            None => AggregateGroupDeltaAuthority::transition(scheme, &EmptyGroupState, scope),
        }
        .map_err(refuse)?;
        if !transition.is_noop() {
            result.push(CommittedGroupTransition {
                kind: GroupTransitionKind::Partition,
                scheme: scheme.clone(),
                transition,
            });
        }
    }
    Ok(result)
}

fn member_key(
    member: &NativeMemberKey,
    find: impl FnOnce(
        contracts::identity::data::record_ids::EntityId,
    ) -> Option<contracts::lineage::data::PersistentNameIdentity>,
) -> Result<CanonicalGroupMemberIdentity, Denial> {
    let carrier = find(member.carrier()).ok_or_else(|| Denial::UnnamedMember {
        member: member.clone(),
    })?;
    Ok(CanonicalGroupMemberIdentity::for_member(carrier, member))
}
