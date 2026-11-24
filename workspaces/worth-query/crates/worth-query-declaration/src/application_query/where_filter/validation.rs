//! Definition-level admission of a where filter against its query.
use super::{ApplicationQueryWhereFilterProgram, ApplicationQueryWhereFilterSource as Source};
use crate::application_query::{
    ApplicationQueryDefinitionDenial as Denial, ApplicationQueryParameterDefinition,
    WorthQueryPortableApplicationQueryParts,
};

/// O(log P) parameter roster lookup using sorted binary search. Reuses an already-sorted
/// canonical parameter slice directly with zero allocation, or sorts borrowed
/// references once in O(P log P) when unsorted.
pub(in crate::application_query) enum ParameterLookup<'a> {
    Sorted(&'a [ApplicationQueryParameterDefinition]),
    Indexed(Vec<&'a ApplicationQueryParameterDefinition>),
}

impl<'a> ParameterLookup<'a> {
    pub(in crate::application_query) fn new(
        parameters: &'a [ApplicationQueryParameterDefinition],
    ) -> Self {
        if parameters
            .windows(2)
            .all(|pair| pair[0].name() <= pair[1].name())
        {
            Self::Sorted(parameters)
        } else {
            let mut indexed: Vec<&'a ApplicationQueryParameterDefinition> =
                parameters.iter().collect();
            indexed.sort_unstable_by_key(|param| param.name());
            Self::Indexed(indexed)
        }
    }

    pub(in crate::application_query) fn get(
        &self,
        name: &str,
    ) -> Option<&'a ApplicationQueryParameterDefinition> {
        match self {
            Self::Sorted(params) => params
                .binary_search_by_key(&name, |param| param.name())
                .ok()
                .map(|idx| &params[idx]),
            Self::Indexed(params) => params
                .binary_search_by_key(&name, |param| param.name())
                .ok()
                .map(|idx| params[idx]),
        }
    }
}

/// Verifies every parameter operand against the query's declared parameter list
/// in O(T log P) time: existence, exact scalar family, unitless discipline, and
/// exact optionality.
pub(in crate::application_query) fn validate_parameter_operands(
    parameters: &[ApplicationQueryParameterDefinition],
    filter: &ApplicationQueryWhereFilterProgram,
) -> Result<(), Denial> {
    let lookup = ParameterLookup::new(parameters);
    validate_parameter_operands_with_lookup(&lookup, filter)
}

pub(in crate::application_query) fn validate_parameter_operands_with_lookup(
    lookup: &ParameterLookup<'_>,
    filter: &ApplicationQueryWhereFilterProgram,
) -> Result<(), Denial> {
    for operand in filter.operands() {
        match operand.source() {
            Source::Parameter {
                parameter,
                scalar_family,
            }
            | Source::OptionalParameter {
                parameter,
                scalar_family,
            } => {
                let Some(declared) = lookup.get(parameter) else {
                    return Err(Denial::WhereFilterUnknownParameter);
                };
                if declared.scalar_family() != *scalar_family
                    || declared.unit().is_some()
                    || declared.is_optional()
                        != matches!(operand.source(), Source::OptionalParameter { .. })
                {
                    return Err(Denial::WhereFilterParameterMismatch);
                }
            }
            Source::Field { .. } | Source::Literal { .. } => {}
        }
    }
    Ok(())
}

/// The filter reads only the root entity's own fields and declared
/// parameters of the same scalar family, and narrows only the selection
/// shapes whose candidates are root rows: the exact scope root and an
/// installed root-path union.
pub(in crate::application_query) fn validate(
    definition: &WorthQueryPortableApplicationQueryParts,
) -> Result<(), Denial> {
    let Some(filter) = definition.where_filter() else {
        return Ok(());
    };
    if definition.orbit().is_some()
        || definition.component_selection().is_some()
        || definition.managed_index().is_some()
        || definition.continuation().is_some()
    {
        return Err(Denial::WhereFilterUnsupportedShape);
    }
    for operand in filter.operands() {
        if let Source::Field { entity, .. } = operand.source()
            && entity != definition.root_entity()
        {
            return Err(Denial::WhereFilterFieldOutsideRoot);
        }
    }
    validate_parameter_operands(definition.parameters(), filter)
}
