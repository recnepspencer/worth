//! A projected selector is resolved only by the exact pinned Native schema/view.
use super::*;
use crate::aggregate_identity::projected_storage::domain_for_contract;
use crate::schema::data::{AggregateIdentityContractDeclaration, RelationalSchemaRegistry};

pub(super) fn validate(
    view: &InvariantStateView<'_>,
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    member: &NativeMemberKey,
    work: &mut funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    let domain = domain_for_contract(registry, contract, work)?;
    match (&domain, member) {
        (None, NativeMemberKey::Entity(_)) => (),
        (Some(domain), NativeMemberKey::Projected(member)) => {
            let names = member
                .domain
                .name_bytes()
                .and_then(|n| n.checked_add(member.case.len()))
                .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
            let levels = u64::from(usize::BITS - domain.parts().cases.len().max(1).leading_zeros());
            work.checkpoint(
                (names as u64)
                    .checked_mul(levels)
                    .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
            )?;
            if member.domain != domain.parts().identity || domain.case(&member.case).is_none() {
                return work.invalid(PlanDenial::WrongMemberKind);
            }
        }
        _ => return work.invalid(PlanDenial::WrongMemberKind),
    }
    let Some(actual) = view.entity_metadata(member.carrier()) else {
        return work.invalid(PlanDenial::MemberUnavailable);
    };
    work.acquired(Acquisition::Entity)?;
    if actual.kind_id != contract.member_kind {
        return work.invalid(PlanDenial::WrongMemberKind);
    }
    Ok(())
}
