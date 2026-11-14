//! Discovery input data survives reopen; every read uses fresh installed authority.
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial as Denial, WorthQueryOutputDemandDenialKind as Kind,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{
        fragment::RecordSelection, ApplicationScalarValueBinding, ApplicationSchema,
    },
};
mod read;
#[derive(Clone)]
pub struct WorthQueryOutputDiscoveryRecipe<D> {
    original: Original<D>,
}
#[derive(Clone)]
enum Original<D> {
    Live(D),
    Recorded(RecordedDiscoveryQuery),
}
impl<D> From<D> for WorthQueryOutputDiscoveryRecipe<D> {
    fn from(value: D) -> Self {
        Self {
            original: Original::Live(value),
        }
    }
}
impl<D> WorthQueryOutputDiscoveryRecipe<D> {
    pub fn live(&self) -> Option<&D> {
        match &self.original {
            Original::Live(value) => Some(value),
            Original::Recorded(_) => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn readmitted(
        bytes: &[u8],
    ) -> Result<Self, Denial> {
        Ok(Self {
            original: Original::Recorded(rmp_serde::from_slice(bytes).map_err(|_| unavailable())?),
        })
    }
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct RecordedDiscoveryQuery {
    query: [u8; 32],
    parameters: Vec<(String, AspectValue)>,
    field: (String, String, String),
    scope: Option<AspectValue>,
}
pub(in crate::domain_computation::primary_graph) fn capture<S, B>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    input: B::Input,
) -> Result<
    Vec<u8>,
    crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    B::Input: ApplicationQueryIntent<S, Binding = B>,
    B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
{
    use crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial as Refusal;
    type Field<S, B> =
        <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Field;
    let parameters =
        crate::domain_computation::primary_graph::application_query::capture_parameters::<S, B>(
            execution,
            runtime,
            0,
            || Ok(input.parameters()),
        )
        .map_err(|_| Refusal::MalformedFrame)?;
    let (field, value) = input
        .into_scope()
        .into_request_parts()
        .ok_or(Refusal::MalformedFrame)?;
    let expected = B::scope_field();
    if (field.entity(),field.aspect(),field.field()) != (expected.entity(),expected.aspect(),expected.field())
        || value.is_none() != matches!(<B::ScopeBinding as ApplicationQueryScopeBinding<S>>::RESOLUTION_MODE, worth_query_declaration::facade::application_query::ApplicationQueryScopeResolutionMode::PrincipalIdentity) { return Err(Refusal::MalformedFrame); }
    let scope = value
        .as_ref()
        .map(|value| {
            <Field<S, B> as RecordSelection>::Binding::encode(value)
                .map_err(|_| Refusal::MalformedFrame)
        })
        .transpose()?;
    let row = RecordedDiscoveryQuery {
        query: *parameters.query().as_bytes(),
        parameters: parameters
            .parameters()
            .bindings()
            .iter()
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect(),
        field: (
            field.entity().to_owned(),
            field.aspect().to_owned(),
            field.field().to_owned(),
        ),
        scope,
    };
    rmp_serde::to_vec_named(&row).map_err(|_| Refusal::MalformedFrame)
}
fn unavailable() -> Denial {
    Denial::new(
        Kind::RetainedBasisUnavailable,
        "recorded discovered output query",
    )
}
