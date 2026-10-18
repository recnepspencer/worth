//! Admission of fresh bindings before their clone, sort and canonical preparation.
use super::*;
use crate::application_query::parameter_canonical_basis::{
    prepare_parameter_basis_admitted, ParameterBasisPreparationStop,
};

#[derive(Debug)]
pub enum WorthQueryParameterPreparationStop<Stop> {
    Admission(Stop),
    Parameters(WorthQueryApplicationQueryParameterDenial),
    AccountingOverflow,
}

pub fn admit_application_query_parameters_admitted<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Stop,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    parameters: ApplicationQueryParameterSet<Query>,
    mut admit: impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<WorthQueryAdmittedApplicationQueryParameters, WorthQueryParameterPreparationStop<Stop>>
{
    use WorthQueryParameterPreparationStop::{AccountingOverflow, Admission, Parameters};
    admit(2, 0).map_err(Admission)?;
    let values = parameters.bindings();
    let slots = values
        .len()
        .checked_mul(std::mem::size_of::<(&'static str, AspectValue)>())
        .ok_or(AccountingOverflow)?;
    admit(values.len(), slots).map_err(Admission)?;
    for (name, value) in values {
        admit(2, 0).map_err(Admission)?;
        let work = name
            .len()
            .checked_add(value.semantic_byte_width())
            .ok_or(AccountingOverflow)?;
        admit(work, value.owned_allocation_capacity_bytes()).map_err(Admission)?;
    }
    // Unstable sort needs no scratch; a unique, validated parameter name has
    // exactly the same ordinary ordering. Charge a conservative quadratic
    // comparison bound, including variable names, before sorting.
    admit(query.parameters().len(), 0).map_err(Admission)?;
    let width = values
        .iter()
        .map(|(name, _)| name.len())
        .chain(
            query
                .parameters()
                .iter()
                .map(|parameter| parameter.name().len()),
        )
        .chain(std::iter::once(query.name().len()))
        .max()
        .unwrap_or(0);
    let comparisons = values
        .len()
        .checked_mul(values.len())
        .and_then(|n| n.checked_add(1))
        .and_then(|n| n.checked_mul(width.checked_add(1)?))
        .ok_or(AccountingOverflow)?;
    let lookup = query
        .parameters()
        .len()
        .checked_mul(values.len().checked_add(1).ok_or(AccountingOverflow)?)
        .and_then(|n| n.checked_mul(width.checked_add(1)?))
        .ok_or(AccountingOverflow)?;
    admit(
        comparisons
            .checked_add(lookup)
            .and_then(|work| work.checked_add(width.checked_mul(2)?))
            .ok_or(AccountingOverflow)?,
        width.checked_mul(2).ok_or(AccountingOverflow)?,
    )
    .map_err(Admission)?;
    let mut bindings = values.to_vec();
    bindings.sort_unstable_by_key(|(name, _)| *name);
    validate_parameter_layout(query, &bindings).map_err(Parameters)?;
    let canonical = prepare_parameter_basis_admitted(
        &bindings,
        query.canonical_work_policy().parameters(),
        &mut admit,
    )
    .map_err(|stop| match stop {
        ParameterBasisPreparationStop::Admission(stop) => Admission(stop),
        ParameterBasisPreparationStop::Derivation(stop) => {
            Parameters(canonical_work_denial(query.name(), stop))
        }
        ParameterBasisPreparationStop::AccountingOverflow => AccountingOverflow,
    })?;
    Ok(WorthQueryAdmittedApplicationQueryParameters {
        canonical,
        bindings,
        budget: query.canonical_work_policy().parameters(),
    })
}
