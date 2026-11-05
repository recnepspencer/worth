//! Moved whole from `domain_computation::primary_graph::application_invariant::refusal`: the items below and every impl they own.
use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;

pub(in crate::domain_computation::primary_graph) fn maximum<Binding: ApplicationScalarValueBinding>() -> Result<usize, String> {
    if Binding::SCALAR_FAMILY != ScalarAspectType::String {
        return Err("invariant refusal requires a canonical application value binding".into());
    }
    canonical_wire_maximum::<Binding>()
        .ok_or_else(|| "invariant refusal requires a finite canonical wire bound".into())
}

/// The finite, nonzero wire bound of a canonical String value binding. Every
/// declared domain value (invariant refusal or producer denial) needs one.
pub(crate) fn canonical_wire_maximum<Binding: ApplicationScalarValueBinding>() -> Option<usize> {
    if Binding::SCALAR_FAMILY != ScalarAspectType::String {
        return None;
    }
    Binding::canonical_contract()
        .and_then(|contract| contract.bounds().maximum_wire_bytes())
        .filter(|maximum| *maximum > 0)
}
