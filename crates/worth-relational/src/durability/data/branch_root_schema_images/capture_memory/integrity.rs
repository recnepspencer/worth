//! Relation-integrity declarations own their full contract IDs and kind rosters.
use super::*;

pub(super) fn quote(
    memory: &mut Quote<'_, '_, '_>,
    declarations: &RelationIntegrityDeclarations,
) -> Result<(), RelationalRecoveryStop> {
    macro_rules! roster {
        ($field:ident) => {{
            // Vec::clone copies exactly len slots; each owned contract ID is
            // separately copied and never hidden in an inline-slot estimate.
            let values = &declarations.$field;
            quote_roster(memory, values, |item| &item.contract_id)?;
        }};
    }
    roster!(endpoint_kind_contracts);
    for declaration in &declarations.endpoint_kind_contracts {
        memory.array::<KindId>(declaration.allowed_source_kinds.len())?;
        memory.array::<KindId>(declaration.allowed_target_kinds.len())?;
    }
    roster!(cardinality_contracts);
    roster!(uniqueness_contracts);
    roster!(symmetry_contracts);
    roster!(endpoint_deletion_integrity_contracts);
    roster!(acyclicity_contracts);
    roster!(partition_isolation_contracts);
    roster!(connectivity_minimum_contracts);
    for declaration in &declarations.connectivity_minimum_contracts {
        memory.array::<KindId>(declaration.source_kind_ids.len())?;
        memory.array::<KindId>(declaration.target_kind_ids.len())?;
    }
    if let Some(aggregate) = &declarations.aggregate_identity {
        use crate::schema::data::AggregateIdentitySource as Source;
        match &aggregate.source {
            Source::Orbit(_) => {}
            Source::Components { relations } => memory.array::<KindId>(relations.len())?,
            Source::DerivedOrbit { derived_orbit } => {
                memory.text(&derived_orbit.schema.0)?;
                memory.text(&derived_orbit.name)?;
            }
            Source::DerivedComponents { derived_components } => {
                memory.text(&derived_components.schema.0)?;
                memory.text(&derived_components.name)?;
            }
        }
    }
    Ok(())
}

fn quote_roster<T>(
    memory: &mut Quote<'_, '_, '_>,
    values: &[T],
    identity: impl Fn(&T) -> &crate::schema::data::ContractId,
) -> Result<(), RelationalRecoveryStop> {
    memory.array::<T>(values.len())?;
    for value in values {
        memory.text(identity(value).as_str())?;
    }
    Ok(())
}
