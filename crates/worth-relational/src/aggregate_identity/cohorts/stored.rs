//! One complete stored-row/membership check for both physical and logical groups.
use super::*;

pub(in crate::aggregate_identity) fn read_stored_group(
    view: &InvariantStateView<'_>,
    members: &BTreeSet<Member>,
    contract: &AggregateIdentityContractDeclaration,
    registry: &RelationalSchemaRegistry,
    memberships: &mut BTreeMap<Member, (RelationId, AggregateIdentity)>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<StoredAggregate> {
    let Some(first) = members.first() else {
        return work.invalid(PlanDenial::InvalidPermutation);
    };
    let mut carriers = BTreeMap::new();
    let first_bindings =
        super::super::membership::read(view, registry, contract, first.carrier(), work)?;
    super::super::membership::key_lookup(first, first_bindings.len(), work)?;
    let Some(&(_, entity)) = first_bindings.get(first) else {
        return work.invalid(PlanDenial::IncompleteStoredMembership);
    };
    work.retain(extent::tree_entry::<
        EntityId,
        BTreeMap<Member, (RelationId, EntityId)>,
    >())?;
    carriers.insert(first.carrier(), first_bindings);
    let row = super::super::stored_record::read(view, entity, contract, registry, work)?;
    super::super::membership::key_lookup(first, 1, work)?;
    if row.current_anchor.as_ref() != Some(first) || row.retired.is_some() {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    for member in members {
        work.checkpoint(1)?;
        if !carriers.contains_key(&member.carrier()) {
            let bindings =
                super::super::membership::read(view, registry, contract, member.carrier(), work)?;
            work.retain(extent::tree_entry::<
                EntityId,
                BTreeMap<Member, (RelationId, EntityId)>,
            >())?;
            carriers.insert(member.carrier(), bindings);
        }
        let bindings = &carriers[&member.carrier()];
        super::super::membership::key_lookup(member, bindings.len(), work)?;
        let Some(&(relation, owner)) = bindings.get(member) else {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        };
        if owner != entity {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        }
        super::super::membership::key_lookup(member, memberships.len(), work)?;
        work.retain(extent::tree_entry::<Member, (RelationId, AggregateIdentity)>())?;
        if memberships
            .insert(member.clone(), (relation, row.identity))
            .is_some()
        {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        }
    }
    // A grouped edge cannot smuggle another case into this cohort's owner.
    for bindings in carriers.values() {
        for (member, &(_, owner)) in bindings {
            work.checkpoint(1)?;
            super::super::membership::key_lookup(member, members.len(), work)?;
            if owner == entity && !members.contains(member) {
                return work.invalid(PlanDenial::IncompleteStoredMembership);
            }
        }
    }
    Ok(StoredAggregate {
        entity,
        record: row,
    })
}
