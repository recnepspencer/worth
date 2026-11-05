//! Read the existing transition; private physical consequences are not applied early.
use super::*;
use crate::schema::data::AggregateIdentityContractDeclaration;
use std::collections::BTreeSet;

pub(super) fn read(
    candidate: &super::super::AggregateCandidateRead,
    contract: &AggregateIdentityContractDeclaration,
    member: &NativeMemberKey,
    work: &mut funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Option<result::Value>> {
    let definitions = &candidate.projection.data.definitions;
    work.checkpoint(definitions.len() as u64 + 1)?;
    let Some(definition) = definitions.get(&contract.definition) else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    let Some(changed) = &definition.changed else {
        return Ok(None);
    };
    super::super::membership::key_lookup(
        member,
        changed
            .member_assignments
            .len()
            .saturating_add(changed.prior_memberships.len()),
        work,
    )?;
    let Some(&index) = changed.member_assignments.get(member) else {
        if changed.prior_memberships.contains_key(member) {
            return work.invalid(PlanDenial::MemberUnavailable);
        }
        return Ok(None);
    };
    let Some(assignment) = changed.transition.assignments.get(index) else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    if !assignment.members.contains(member) {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    let mut lineage = BTreeSet::new();
    if !assignment.born {
        let mut prior_member = None;
        for (member, (_, identity)) in &changed.prior_memberships {
            work.checkpoint(1)?;
            if *identity == assignment.identity {
                prior_member = Some(member.clone());
                break;
            }
        }
        let Some(prior_member) = prior_member else {
            return work.invalid(PlanDenial::InvalidStoredIdentity);
        };
        let before = RelationalBranchRootState::new(std::sync::Arc::clone(&candidate.before));
        let view = InvariantStateView::new(&before, candidate.before_version);
        let registry = candidate.before.schema_authority().registry();
        let group =
            super::super::stored_read::collect(&view, contract, registry, prior_member, work)?;
        if group.stored.record.identity != assignment.identity {
            return work.invalid(PlanDenial::InvalidStoredIdentity);
        }
        lineage =
            super::super::stored_lineage::read(&view, &group.stored, contract, registry, work)?;
    }
    for event in &changed.transition.lineage {
        work.checkpoint(1)?;
        if event.from == assignment.identity || event.to == assignment.identity {
            work.retain(extent::tree_entry::<super::super::AggregateLineageEvent, ()>())?;
            lineage.insert(*event);
        }
    }
    let mut members = BTreeSet::new();
    for member in &assignment.members {
        work.checkpoint(1)?;
        work.retain(extent::tree_entry::<NativeMemberKey, ()>())?;
        members.insert(member.clone());
    }
    Ok(Some(result::Value {
        definition: contract.definition,
        identity: assignment.identity,
        members,
        lineage,
    }))
}
