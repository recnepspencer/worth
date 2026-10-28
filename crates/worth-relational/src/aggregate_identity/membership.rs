//! Strict physical edges and grouped logical cases on real carrier relations.
use super::funding::{AggregateWork, BuildResult, PlanDenial};
use super::observations::Acquisition;
use super::projected_storage::{decode_cases, domain_for_contract, member_key, MEMBERSHIP_ASPECT};
use crate::graph_member::NativeMemberKey as Member;
use crate::identity::data::{EntityId, RelationId};
use crate::schema::data::{AggregateIdentityContractDeclaration, RelationalSchemaRegistry};
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey, InternedString,
};

/// One carrier may own different cohorts for its cases. Every complete logical
/// key occurs on exactly one edge; a single (carrier,target) edge holds all its
/// cases, in the registered canonical roster order.
pub(super) fn read(
    view: &InvariantStateView<'_>,
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    carrier: EntityId,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<BTreeMap<Member, (RelationId, EntityId)>> {
    let domain = domain_for_contract(registry, contract, work)?;
    let mut bindings = BTreeMap::new();
    let mut owners = BTreeSet::new();
    for id in super::cohorts::outgoing(view, carrier, work)? {
        work.checkpoint(1)?;
        let Some(edge) = view.relation_metadata(id) else {
            continue;
        };
        work.acquired(Acquisition::Relation)?;
        if edge.kind_id != contract.membership_kind {
            continue;
        }
        work.retain(super::retention_extent::tree_entry::<EntityId, ()>())?;
        if edge.source != carrier || !owners.insert(edge.target) {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        }
        let state = view.relation_aspect_state(id);
        let aspect = state.and_then(|state| {
            state.get(&AspectKey::new(MEMBERSHIP_ASPECT).expect("fixed membership aspect"))
        });
        work.acquired(Acquisition::Aspect)?;
        let Some(domain) = &domain else {
            work.retain(super::retention_extent::tree_entry::<
                Member,
                (RelationId, EntityId),
            >())?;
            if aspect.is_some()
                || bindings
                    .insert(Member::Entity(carrier), (id, edge.target))
                    .is_some()
            {
                return work.invalid(PlanDenial::IncompleteStoredMembership);
            }
            continue;
        };
        let Some(aspect) = aspect else {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        };
        let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        };
        if fields.len() != 1 {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        }
        let Some(AspectValue::String(InternedString::Raw(cases))) =
            fields.get(&FieldKey::new("cases").expect("fixed case field"))
        else {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        };
        for case in decode_cases(cases, domain, work)? {
            let key = member_key(domain, carrier, case, work)?;
            key_lookup(&key, bindings.len(), work)?;
            work.retain(super::retention_extent::tree_entry::<
                Member,
                (RelationId, EntityId),
            >())?;
            if bindings.insert(key, (id, edge.target)).is_some() {
                return work.invalid(PlanDenial::IncompleteStoredMembership);
            }
        }
    }
    Ok(bindings)
}

pub(super) fn key_lookup(
    member: &Member,
    count: usize,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    let units = member
        .comparison_work(count)
        .and_then(|units| u64::try_from(units).ok())
        .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
    if units != 0 {
        work.checkpoint(units)?;
    }
    Ok(())
}
