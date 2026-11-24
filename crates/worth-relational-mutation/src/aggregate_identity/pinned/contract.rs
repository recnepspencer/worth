//! Resolve only the actual selected root's aggregate contract.
use super::*;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_contracts::internal_port::schema::data::relation_integrity::aggregate::AggregateIdentityContractDeclaration;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

pub(super) fn resolve<'root>(
    root: &'root RelationalBranchRoot,
    selection: crate::aggregate_identity::selection::AggregateSelection,
    work: &mut super::super::funding::AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<&'root AggregateIdentityContractDeclaration> {
    let mut selected = None;
    for kind in (*root.schema_authority().registry().owner_relation_kinds()).values() {
        work.checkpoint(1)?;
        if let Some(found) = kind
            .relation_integrity
            .aggregate_identity
            .as_ref()
            .filter(|contract| selection.matches(contract))
            && selected.replace(found).is_some()
        {
            return work.invalid(PlanDenial::AmbiguousAggregateRelation);
        }
    }
    selected.ok_or(MapKernelFailure::Domain(if selection.is_definition() {
        PlanDenial::DefinitionUnavailable
    } else {
        PlanDenial::NotAggregateRelation
    }))
}
