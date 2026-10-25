//! Exact recovery cannot replace the laws of a current branch head.
use super::recovered_codec_meaning::{charge, lookup_work};
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::schema::data::{RelationIntegrityDeclarations, RelationalSchemaRegistry};
use worth_execution::MapKernelContext;

pub(super) fn compare_registries(
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    for (kind, previous) in &source.relation_kinds {
        charge(context, lookup_work(target.relation_kinds.len()))?;
        let previous = &previous.relation_integrity;
        charge_meaning(context, previous)?;
        let next = target
            .relation_kinds
            .get(kind)
            .map(|kind| &kind.relation_integrity);
        let same = if let Some(next) = next {
            charge_meaning(context, next)?;
            previous.acyclicity_contracts == next.acyclicity_contracts
                && previous.symmetry_contracts == next.symmetry_contracts
                && previous.uniqueness_contracts == next.uniqueness_contracts
                && previous.partition_isolation_contracts == next.partition_isolation_contracts
                && previous.connectivity_minimum_contracts == next.connectivity_minimum_contracts
        } else {
            previous.acyclicity_contracts.is_empty()
                && previous.symmetry_contracts.is_empty()
                && previous.uniqueness_contracts.is_empty()
                && previous.partition_isolation_contracts.is_empty()
                && previous.connectivity_minimum_contracts.is_empty()
        };
        if !same {
            return Err(DurabilityError::new(
                RecoveryFailureClass::SchemaMismatch,
                "recovered current relation laws differ from target schema meaning",
            )
            .into());
        }
    }
    Ok(())
}

fn charge_meaning(
    context: &mut MapKernelContext<'_, '_>,
    laws: &RelationIntegrityDeclarations,
) -> Result<(), RelationalRecoveryStop> {
    for id in laws
        .acyclicity_contracts
        .iter()
        .map(|law| &law.contract_id)
        .chain(laws.symmetry_contracts.iter().map(|law| &law.contract_id))
        .chain(laws.uniqueness_contracts.iter().map(|law| &law.contract_id))
        .chain(
            laws.partition_isolation_contracts
                .iter()
                .map(|law| &law.contract_id),
        )
        .chain(
            laws.connectivity_minimum_contracts
                .iter()
                .map(|law| &law.contract_id),
        )
    {
        charge(context, 1)?;
        charge(context, id.as_str().len())?;
    }
    for law in &laws.connectivity_minimum_contracts {
        charge(context, law.source_kind_ids.len())?;
        charge(context, law.target_kind_ids.len())?;
    }
    Ok(())
}
