//! Temporal re-entry consumes real current rows and fresh authentication.
use super::{
    authentication,
    composite_definition_contract::ReadyNode,
    composite_definition_model::{Change, Tick},
    composite_definition_query::Row,
    composite_definition_runtime::Clock,
    schema::*,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter,
    declaration::application_schema::{StringApplicationValueBinding, TypedMutationPreconditions},
    domain,
    primary_graph::*,
};
pub struct IntentProjector;
impl domain::WorthQueryTemporalIntentProjector<ReadyNode, Clock, Row, Change> for IntentProjector {
    const SEMANTIC_IDENTITY: &'static str = "numeric-definition.projector.v1";
    fn project(
        &self,
        row: &Row,
    ) -> Result<
        domain::WorthQueryTemporalIntentCandidate<Clock, Change>,
        domain::WorthQueryTemporalIntentProjectionFailure,
    > {
        let identity = domain::WorthQueryTemporalIntentIdentity::declare(row.key.clone())
            .map_err(projection_failure)?;
        let input_identity = domain::WorthQueryTemporalOperationInputIdentity::declare(format!(
            "{}:{}",
            row.key, row.revision
        ))
        .map_err(projection_failure)?;
        let relation =
            domain::WorthQueryTemporalIntentIdempotencyRelation::declare(&(&row.key, row.revision))
                .map_err(|cause| projection_failure(format!("{cause:?}")))?;
        let due = domain::WorthQueryClockCoordinate::from_nanoseconds(row.due);
        let input = Change {
            key: row.key.clone(),
            value: super::composite_definition_runtime::TICK_OUTPUT,
        };
        let candidate = if row.lifecycle == 100 {
            domain::WorthQueryTemporalIntentCandidate::active::<StringApplicationValueBinding>(
                identity,
                row.key.clone(),
                row.revision,
                due,
                input,
                input_identity,
                relation,
            )
        } else {
            domain::WorthQueryTemporalIntentCandidate::completed::<StringApplicationValueBinding>(
                identity,
                row.key.clone(),
                row.revision,
                due,
                input,
                input_identity,
                relation,
            )
        };
        candidate.map_err(|cause| projection_failure(format!("{cause:?}")))
    }
}
fn projection_failure(
    detail: impl Into<String>,
) -> domain::WorthQueryTemporalIntentProjectionFailure {
    domain::WorthQueryTemporalIntentProjectionFailure::new(
        domain::WorthQueryTemporalIntentProjectionFailureKind::InvalidIdentity,
        detail,
    )
}
pub struct PrincipalSource(
    pub WorthQueryAdmittedAuthenticationAdapter<NumericSchema, authentication::OperatorAdapter>,
);
impl WorthQueryTemporalPrincipalSource<NumericSchema> for PrincipalSource {
    const SEMANTIC_IDENTITY: &'static str = "numeric-definition.principal.v1";
    fn admit(
        &self,
    ) -> Result<
        WorthQueryTemporalPrincipalAdmission<NumericSchema>,
        WorthQueryTemporalPrincipalFailure,
    > {
        let scope = authentication::request_scope();
        let principal =
            authentication::block_on(self.0.authenticate((), &scope)).map_err(|cause| {
                WorthQueryTemporalPrincipalFailure::new(
                    WorthQueryTemporalPrincipalFailureKind::AdmissionRejected,
                    format!("{cause:?}"),
                )
            })?;
        Ok(WorthQueryTemporalPrincipalAdmission::new(principal, scope))
    }
}
pub struct Invoker;
impl WorthQueryTemporalOperationInvoker<NumericSchema, Tick, Change, Number> for Invoker {
    const SEMANTIC_IDENTITY: &'static str = "numeric-definition.invoker.v1";
    type Projection = WorthQueryInvariantMutationTarget<NumericSchema, Number>;
    fn preconditions(&self, _: &Change) -> TypedMutationPreconditions<NumericSchema, Tick, Number> {
        Default::default()
    }
    fn project(
        &self,
        _: &Change,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            NumericSchema,
            Tick,
        >,
        scope: &WorthQueryInvariantEntityIdentity<NumericSchema, Number>,
    ) -> Result<Self::Projection, WorthQueryTemporalInvocationFailure> {
        reader
            .require_decision_field(scope, Key::reference())
            .map_err(invocation_failure)?;
        reader
            .decision_field(scope, Secondary::reference())
            .map_err(invocation_failure)?;
        reader.mutation_target(scope).map_err(invocation_failure)
    }
    fn apply(
        &self,
        input: Change,
        target: Self::Projection,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<
            NumericSchema,
            Tick,
            Change,
            Number,
        >,
    ) -> Result<(), WorthQueryTemporalInvocationFailure> {
        let target = effects
            .projected_entity(&target)
            .map_err(invocation_failure)?;
        effects
            .write_field(&target, Secondary::reference(), input.value)
            .map_err(invocation_failure)
    }
}
fn invocation_failure(cause: impl std::fmt::Display) -> WorthQueryTemporalInvocationFailure {
    WorthQueryTemporalInvocationFailure::new(
        WorthQueryTemporalInvocationFailureKind::InvocationRejected,
        cause.to_string(),
    )
}
