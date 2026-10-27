//! Strict optional coordinates bind row carriers to the actual registered domain.
use super::{AggregateRecord, AggregateWork, BuildResult, PlanDenial};
use crate::aggregate_identity::projected_storage::{
    decode_domain, domain_for_contract, COORDINATE_ASPECT,
};
use crate::schema::data::{AggregateIdentityContractDeclaration, RelationalSchemaRegistry};
use worth_foundational::facade::{
    AspectKey, AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
    FieldKey, InternedString,
};

pub(super) fn bind(
    row: &mut AggregateRecord,
    state: &AuthoritativeRecordAspectState,
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    let coordinate =
        state.get(&AspectKey::new(COORDINATE_ASPECT).expect("fixed coordinate aspect"));
    let Some(domain) = domain_for_contract(registry, contract, work)? else {
        return if coordinate.is_none() {
            Ok(())
        } else {
            work.invalid(PlanDenial::InvalidStoredIdentity)
        };
    };
    if contract.storage.projected.is_none() {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    let Some(coordinate) = coordinate else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    let ContractValidatedAspectValueView::Struct(fields) = coordinate.view() else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    if fields.len() != 3 {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    let text = |name| match fields.get(&FieldKey::new(name).expect("fixed coordinate field")) {
        Some(AspectValue::String(InternedString::Raw(value))) => Some(value.as_str()),
        _ => None,
    };
    let invalid = || worth_execution::MapKernelFailure::Domain(PlanDenial::InvalidStoredIdentity);
    let encoded = text("domain").ok_or_else(invalid)?;
    if !decode_domain(encoded, &domain.parts().identity, work)?.is_empty() {
        return Err(invalid());
    }
    let seed = text("seed-case").ok_or_else(invalid)?;
    let current = text("current-case").ok_or_else(invalid)?;
    work.checkpoint(
        (seed.len() as u64)
            .checked_add(current.len() as u64)
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    if row.current_anchor.is_none() && seed != current {
        return Err(invalid());
    }
    row.seed = crate::aggregate_identity::projected_storage::member_key(
        &domain,
        row.seed.carrier(),
        seed,
        work,
    )?;
    if let Some(anchor) = &row.current_anchor {
        row.current_anchor = Some(crate::aggregate_identity::projected_storage::member_key(
            &domain,
            anchor.carrier(),
            current,
            work,
        )?);
    }
    Ok(())
}
