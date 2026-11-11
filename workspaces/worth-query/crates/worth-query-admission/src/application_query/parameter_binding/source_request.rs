//! Immutable normalized selector data shares the admitted parameter canonical engine.
use super::*;
use crate::application_query::parameter_canonical_basis::{
    prepare_parameter_basis_admitted, ParameterBasisPreparationStop,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;

/// Request data and canonical evidence only; this carries no principal or grant.
pub struct WorthQueryCanonicalSourceRequest {
    query: WorthQueryInstalledApplicationQueryIdentity,
    parameters: CanonicalDigestId,
    field: (&'static str, &'static str, &'static str),
    scope: Option<AspectValue>,
    canonical: WorthQueryApplicationParameterCanonicalArtifact,
}
impl WorthQueryCanonicalSourceRequest {
    pub fn identity(&self) -> &CanonicalDigestId {
        self.canonical.identity()
    }
    pub fn scope(&self) -> Option<&AspectValue> {
        self.scope.as_ref()
    }
    pub fn matches(
        &self,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        parameters: &WorthQueryAdmittedApplicationQueryParameters,
        field: (&str, &str, &str),
    ) -> bool {
        &self.query == query && &self.parameters == parameters.identity() && self.field == field
    }
}
impl WorthQueryAdmittedApplicationQueryParameters {
    /// Prepare once at capture. Readers carry this owner's immutable evidence;
    /// comparing it never reconstructs a canonical basis or derives a digest.
    pub fn capture_source_request_admitted<Schema, Query, Parameters, Result, Scope, Stop>(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, Result, Scope>,
        field: (&'static str, &'static str, &'static str),
        scope: Option<AspectValue>,
        mut admit: impl FnMut(usize, usize) -> std::result::Result<(), Stop>,
    ) -> std::result::Result<
        WorthQueryCanonicalSourceRequest,
        WorthQueryParameterPreparationStop<Stop>,
    > {
        use WorthQueryParameterPreparationStop::{AccountingOverflow, Admission, Parameters};
        let names = field
            .0
            .len()
            .checked_add(field.1.len())
            .and_then(|n| n.checked_add(field.2.len()))
            .ok_or(AccountingOverflow)?;
        let bytes = names
            .checked_add(
                scope
                    .as_ref()
                    .map_or(0, AspectValue::owned_allocation_capacity_bytes),
            )
            .ok_or(AccountingOverflow)?;
        let work = names
            .checked_add(scope.as_ref().map_or(0, AspectValue::semantic_byte_width))
            .and_then(|n| n.checked_add(64))
            .ok_or(AccountingOverflow)?;
        admit(work, bytes).map_err(Admission)?;
        let query_id = query.identity().as_bytes();
        let parameters = self.identity().bytes();
        let bindings = [
            (
                "query.low",
                AspectValue::Uuid(query_id[..16].try_into().expect("query digest half")),
            ),
            (
                "query.high",
                AspectValue::Uuid(query_id[16..].try_into().expect("query digest half")),
            ),
            (
                "parameters.low",
                AspectValue::Uuid(parameters[..16].try_into().expect("parameter digest half")),
            ),
            (
                "parameters.high",
                AspectValue::Uuid(parameters[16..].try_into().expect("parameter digest half")),
            ),
            ("scope.entity", AspectValue::String(field.0.into())),
            ("scope.aspect", AspectValue::String(field.1.into())),
            ("scope.field", AspectValue::String(field.2.into())),
            ("scope.principal", AspectValue::Bool(scope.is_none())),
            ("scope.value", scope.clone().unwrap_or(AspectValue::Null)),
        ];
        // This commits complete request/selector data, not another ordinary
        // parameter row. Actual parameters retain their own narrower policy.
        let canonical = prepare_parameter_basis_admitted(
            &bindings,
            query.canonical_work_policy().admission_planning(),
            &mut admit,
        )
        .map_err(|stop| match stop {
            ParameterBasisPreparationStop::Admission(cause) => Admission(cause),
            ParameterBasisPreparationStop::Preparation { cause, quote } => {
                WorthQueryParameterPreparationStop::Preparation { cause, quote }
            }
            ParameterBasisPreparationStop::Derivation(cause) => {
                Parameters(canonical_work_denial(query.name(), cause))
            }
            ParameterBasisPreparationStop::AccountingOverflow => AccountingOverflow,
            ParameterBasisPreparationStop::UnsupportedSortImplementation(cause) => {
                Parameters(WorthQueryApplicationQueryParameterDenial::new(
                    WorthQueryApplicationQueryParameterDenialKind::UnsupportedSortImplementation(
                        cause,
                    ),
                    query.name(),
                ))
            }
        })?;
        Ok(WorthQueryCanonicalSourceRequest {
            query: query.identity().clone(),
            parameters: *self.identity(),
            field,
            scope,
            canonical,
        })
    }
}
