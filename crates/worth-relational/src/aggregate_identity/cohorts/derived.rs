//! Derived cohorts consume actual before/after sources through the common coordinator.
mod discovery;
mod partition;
use super::*;
use crate::derived_relation::{
    DerivedRelationDenial, DerivedRelationHandle, DerivedRelationReadDenial,
};
use discovery::Discovery;

pub(in crate::aggregate_identity::cohorts) fn collect(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    contract: &AggregateIdentityContractDeclaration,
    registry: &RelationalSchemaRegistry,
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
    effect: &AggregateChanges,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Option<CollectedCohorts>> {
    let handle = handle(registry, contract, work)?;
    let components = matches!(
        contract.source,
        AggregateIdentitySource::DerivedComponents { .. }
    );
    let prepared = super::super::derived_source::with_views(before, after, work, |old, new| {
        let affected = crate::derived_relation::affected::collect(
            registry,
            &handle,
            &effect.adjacency,
            &effect.records,
            old,
            new,
        )?;
        let mut seeds = affected.roots;
        for member in affected.targets {
            discovery::insert(&mut seeds, member, new)?;
        }
        if seeds.is_empty() {
            return Ok(None);
        }
        let prior = discovery::collect(registry, &handle, &seeds, components, old)?;
        // Every retained before cohort participates after severing; direct
        // source endpoints alone are insufficient for the detached fragment.
        for group in &prior.groups {
            for member in group {
                discovery::insert(&mut seeds, member.clone(), new)?;
            }
        }
        let proposed = discovery::collect(registry, &handle, &seeds, components, new)?;
        // Prefix support remains alive with both source discoveries through
        // this complete candidate acquisition, before any identity transition.
        Ok(Some((prior, proposed, affected.support)))
    })?;
    let Some((prior, proposed, _prefix_support)) = prepared else {
        return Ok(None);
    };
    let proposed_groups = if components {
        partition::maintain(execution, &prior, &proposed, work)?
    } else {
        proposed.groups
    };
    let mut collected = CollectedCohorts {
        prior: Vec::new(),
        proposed: proposed_groups,
        rows: BTreeMap::new(),
        memberships: BTreeMap::new(),
    };
    for members in prior.groups {
        let stored = read_stored_group(
            before,
            &members,
            contract,
            registry,
            &mut collected.memberships,
            work,
        )?;
        if collected.rows.contains_key(&stored.record.identity) {
            return work.invalid(PlanDenial::InvalidStoredIdentity);
        }
        collected.prior.push(PriorAggregate {
            identity: stored.record.identity,
            members,
        });
        collected.rows.insert(stored.record.identity, stored);
    }
    Ok(Some(collected))
}

pub(in crate::aggregate_identity) fn groups(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    seeds: &BTreeSet<Member>,
    view: &InvariantStateView<'_>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Vec<BTreeSet<Member>>> {
    let handle = handle(registry, contract, work)?;
    let components = matches!(
        contract.source,
        AggregateIdentitySource::DerivedComponents { .. }
    );
    super::super::derived_source::with_views(view, view, work, |source, _| {
        discovery::collect(registry, &handle, seeds, components, source)
            .map(|discovery| discovery.groups)
    })
}

fn handle(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<DerivedRelationHandle> {
    let Some(identity) = contract.source.derived_relation() else {
        return work.invalid(PlanDenial::DefinitionUnavailable);
    };
    let bytes = identity
        .schema
        .0
        .len()
        .checked_add(identity.name.len())
        .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
    let levels = u64::from(usize::BITS - registry.derived_relations.len().max(1).leading_zeros());
    work.checkpoint(
        (bytes as u64)
            .checked_mul(11)
            .and_then(|n| n.checked_mul(levels))
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    registry
        .derived_relation(identity)
        .ok_or(worth_execution::MapKernelFailure::Domain(
            PlanDenial::DefinitionUnavailable,
        ))
}
