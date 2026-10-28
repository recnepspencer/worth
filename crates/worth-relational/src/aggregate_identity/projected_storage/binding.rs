//! Resolve only the exact selected registry's declared aggregate member domain.
use super::super::funding::{AggregateWork, BuildResult, PlanDenial};
use crate::graph_member::NativeMemberDomain;
use crate::projected_member_domain::ProjectedMemberDomainHandle;
use crate::schema::data::{AggregateIdentityContractDeclaration, RelationalSchemaRegistry};

pub(in crate::aggregate_identity) fn domain_for_contract(
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<Option<ProjectedMemberDomainHandle>> {
    let Some(identity) = contract.source.derived_relation() else {
        return Ok(None);
    };
    let bytes = identity
        .schema
        .0
        .len()
        .checked_add(identity.name.len())
        .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
    lookup(bytes, registry.derived_relations.len(), work)?;
    let Some(recipe) = registry.derived_relations.get(identity) else {
        return work.invalid(PlanDenial::DefinitionUnavailable);
    };
    match &recipe.parts.from {
        NativeMemberDomain::Entity(_) => Ok(None),
        NativeMemberDomain::Projected { projected } => {
            let bytes = projected
                .name_bytes()
                .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
            lookup(bytes, registry.projected_member_domains.len(), work)?;
            registry.projected_member_domain(projected).map(Some).ok_or(
                worth_execution::MapKernelFailure::Domain(PlanDenial::DefinitionUnavailable),
            )
        }
    }
}
fn lookup(bytes: usize, count: usize, work: &mut AggregateWork<'_, '_, '_, '_>) -> BuildResult<()> {
    let levels = u64::from(usize::BITS - count.max(1).leading_zeros());
    let units = u64::try_from(bytes)
        .ok()
        .and_then(|bytes| bytes.checked_mul(11))
        .and_then(|bytes| bytes.checked_mul(levels))
        .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
    work.checkpoint(units)
}

/// Decode a descriptive key only after its case belongs to the actual selected
/// domain. The caller retains the original scope's backing grant; this key is
/// not an observed member or a prepared source proof.
pub(in crate::aggregate_identity) fn member_key(
    domain: &ProjectedMemberDomainHandle,
    carrier: crate::identity::data::EntityId,
    case: &str,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<crate::graph_member::NativeMemberKey> {
    use crate::graph_member::{NativeMemberKey, ProjectedMemberCoordinate};
    let levels = u64::from(usize::BITS - domain.parts().cases.len().max(1).leading_zeros());
    work.checkpoint(
        (case.len() as u64)
            .checked_mul(levels)
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    if domain.case(case).is_none() {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    let bytes = domain
        .parts()
        .identity
        .owned_bytes()
        .checked_add(case.len())
        .and_then(|n| {
            n.checked_add(
                std::mem::size_of::<ProjectedMemberCoordinate>() + 2 * std::mem::size_of::<usize>(),
            )
        })
        .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?;
    work.checkpoint(bytes as u64)?;
    work.retain(bytes as u64)?;
    Ok(NativeMemberKey::Projected(std::sync::Arc::new(
        ProjectedMemberCoordinate {
            domain: domain.parts().identity.clone(),
            carrier,
            case: case.into(),
        },
    )))
}
