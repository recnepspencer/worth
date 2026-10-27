//! Resolve only the actual selected root's aggregate contract.
use super::*;
use crate::branch::RelationalBranchRoot;
use crate::schema::data::AggregateIdentityContractDeclaration;
use worth_foundational::facade::AggregateDefinitionIdentity;

pub(super) fn resolve(
    root: &RelationalBranchRoot,
    relation: KindId,
    definition: Option<AggregateDefinitionIdentity>,
    work: &mut super::super::funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<AggregateIdentityContractDeclaration> {
    let mut selected = None;
    for kind in root.schema_authority().registry().relation_kinds.values() {
        work.checkpoint(1)?;
        if let Some(found) = kind
            .relation_integrity
            .aggregate_identity
            .filter(|contract| {
                contract.source_relation == relation
                    && definition.is_none_or(|id| contract.definition == id)
            })
        {
            if selected.replace(found).is_some() {
                return work.invalid(PlanDenial::AmbiguousAggregateRelation);
            }
        }
    }
    selected.ok_or(MapKernelFailure::Domain(if definition.is_some() {
        PlanDenial::DefinitionUnavailable
    } else {
        PlanDenial::NotAggregateRelation
    }))
}
