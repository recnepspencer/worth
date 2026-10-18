//! Exact installed parameter layout validation for fresh and retained admission.
use super::*;

pub(super) fn validate_parameter_layout<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    bindings: &[(&'static str, AspectValue)],
) -> Result<(), WorthQueryApplicationQueryParameterDenial> {
    if bindings.len() != query.parameters().len() {
        return Err(WorthQueryApplicationQueryParameterDenial::new(
            WorthQueryApplicationQueryParameterDenialKind::ParameterSetMismatch,
            query.name(),
        ));
    }
    for declared in query.parameters() {
        let Some((_, value)) = bindings
            .binary_search_by(|(name, _)| name.cmp(&declared.name()))
            .ok()
            .and_then(|index| bindings.get(index))
        else {
            return Err(WorthQueryApplicationQueryParameterDenial::new(
                WorthQueryApplicationQueryParameterDenialKind::ParameterSetMismatch,
                declared.name(),
            ));
        };
        if value.value_family() != declared.scalar_family() {
            return Err(WorthQueryApplicationQueryParameterDenial::new(
                WorthQueryApplicationQueryParameterDenialKind::ParameterTypeMismatch,
                declared.name(),
            ));
        }
    }
    Ok(())
}
