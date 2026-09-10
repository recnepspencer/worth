mod clock;
mod predicate;
mod transport;

use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_host::facade::{admission, domain, primary_graph};

use super::contract::TemporalReadyNode;
use super::schema::{
    ExecuteTemporal, IntentEffectField, IntentIdentityField, IntentQueryResult,
    TemporalExecutionEffect, TemporalExecutionNotice, TemporalHostSchema, TemporalInput,
    TemporalIntent,
};

pub use clock::{ClockController, ClockSource, ExampleClock};
pub use predicate::{Predicate, ReplacementPredicate};
pub use transport::CompletingExternalTransport;

pub struct IntentProjector;

impl
    domain::WorthQueryTemporalIntentProjector<
        TemporalReadyNode,
        ExampleClock,
        IntentQueryResult,
        TemporalInput,
    > for IntentProjector
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.example.product.projector";

    fn project(
        &self,
        row: &IntentQueryResult,
    ) -> Result<
        domain::WorthQueryTemporalIntentCandidate<ExampleClock, TemporalInput>,
        domain::WorthQueryTemporalIntentProjectionFailure,
    > {
        let identity = domain::WorthQueryTemporalIntentIdentity::declare(row.identity.clone())
            .map_err(projection_failure)?;
        let input_identity =
            domain::WorthQueryTemporalOperationInputIdentity::declare(row.input.clone())
                .map_err(projection_failure)?;
        let idempotency = domain::WorthQueryTemporalIntentIdempotencyRelation::declare(format!(
            "{}:{}:{}",
            row.identity, row.revision, row.input
        ))
        .map_err(projection_failure)?;
        Ok(domain::WorthQueryTemporalIntentCandidate::active(
            identity,
            row.identity.clone(),
            row.revision,
            domain::WorthQueryClockCoordinate::from_nanoseconds(row.due),
            TemporalInput(row.input.clone()),
            input_identity,
            idempotency,
        ))
    }
}

fn projection_failure(detail: &'static str) -> domain::WorthQueryTemporalIntentProjectionFailure {
    domain::WorthQueryTemporalIntentProjectionFailure::new(
        domain::WorthQueryTemporalIntentProjectionFailureKind::InvalidIdentity,
        detail,
    )
}

pub struct Invoker;

impl
    primary_graph::WorthQueryTemporalOperationInvoker<
        TemporalHostSchema,
        ExecuteTemporal,
        TemporalInput,
        TemporalIntent,
    > for Invoker
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.example.product.invoker";
    type Projection = (
        primary_graph::WorthQueryInvariantMutationTarget<TemporalHostSchema, TemporalIntent>,
        String,
    );

    fn preconditions(
        &self,
        _input: &TemporalInput,
    ) -> worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions<
        TemporalHostSchema,
        ExecuteTemporal,
        TemporalIntent,
    > {
        Default::default()
    }

    fn project(
        &self,
        _input: &TemporalInput,
        reader: &mut primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            TemporalHostSchema,
            ExecuteTemporal,
        >,
        scope: &primary_graph::WorthQueryInvariantEntityIdentity<
            TemporalHostSchema,
            TemporalIntent,
        >,
    ) -> Result<Self::Projection, primary_graph::WorthQueryTemporalInvocationFailure> {
        reader
            .require_decision_field(scope, IntentIdentityField::reference())
            .map_err(invocation_projection_failure)?;
        let identity = reader
            .decision_field(scope, IntentIdentityField::reference())
            .map_err(invocation_projection_failure)?
            .ok_or_else(|| {
                primary_graph::WorthQueryTemporalInvocationFailure::new(
                    primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
                    "required temporal identity was absent",
                )
            })?;
        reader
            .decision_field(scope, IntentEffectField::reference())
            .map_err(invocation_projection_failure)?;
        let target = reader.mutation_target(scope).map_err(|detail| {
            primary_graph::WorthQueryTemporalInvocationFailure::new(
                primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
                detail,
            )
        })?;
        Ok((target, identity))
    }

    fn apply(
        &self,
        input: TemporalInput,
        (target, identity): Self::Projection,
        effects: &mut primary_graph::WorthQueryApplicationEffectProgramBuilder<
            TemporalHostSchema,
            ExecuteTemporal,
            TemporalInput,
            TemporalIntent,
        >,
    ) -> Result<(), primary_graph::WorthQueryTemporalInvocationFailure> {
        let target = effects
            .projected_entity(&target)
            .map_err(invocation_execution_failure)?;
        effects
            .write_field(&target, IntentEffectField::reference(), input.0)
            .map_err(invocation_execution_failure)?;
        effects
            .emit(
                TemporalExecutionEffect::reference(),
                TemporalExecutionNotice { identity },
            )
            .map_err(invocation_execution_failure)
    }
}

fn invocation_projection_failure(
    denial: impl ToString,
) -> primary_graph::WorthQueryTemporalInvocationFailure {
    primary_graph::WorthQueryTemporalInvocationFailure::new(
        primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
        denial.to_string(),
    )
}

fn invocation_execution_failure(
    denial: impl ToString,
) -> primary_graph::WorthQueryTemporalInvocationFailure {
    primary_graph::WorthQueryTemporalInvocationFailure::new(
        primary_graph::WorthQueryTemporalInvocationFailureKind::InvocationRejected,
        denial.to_string(),
    )
}

pub struct PrincipalSource {
    adapter: Arc<
        admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
            TemporalHostSchema,
            IdentityAdapter,
        >,
    >,
}

impl PrincipalSource {
    pub fn new(
        adapter: admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
            TemporalHostSchema,
            IdentityAdapter,
        >,
    ) -> Self {
        Self {
            adapter: Arc::new(adapter),
        }
    }
}

impl primary_graph::WorthQueryTemporalPrincipalSource<TemporalHostSchema> for PrincipalSource {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.example.product.principal-source";

    fn admit(
        &self,
    ) -> Result<
        primary_graph::WorthQueryTemporalPrincipalAdmission<TemporalHostSchema>,
        primary_graph::WorthQueryTemporalPrincipalFailure,
    > {
        let scope = request_scope();
        let external = block_on(self.adapter.authenticate((), &scope)).map_err(|denial| {
            primary_graph::WorthQueryTemporalPrincipalFailure::new(
                primary_graph::WorthQueryTemporalPrincipalFailureKind::AdmissionRejected,
                format!("{denial:?}"),
            )
        })?;
        Ok(primary_graph::WorthQueryTemporalPrincipalAdmission::new(
            external, scope,
        ))
    }
}

pub struct IdentityAdapter;

impl admission::authenticated_principal::WorthQueryAuthenticationAdapter for IdentityAdapter {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        "worth-query-product-example-adapter-v1"
    }

    fn validate<'a>(
        &'a self,
        _credential: Self::Credential,
        _scope: &'a admission::authenticated_principal::WorthQueryRequestScope,
    ) -> admission::authenticated_principal::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            admission::authenticated_principal::WorthQueryValidatedExternalPrincipal::new(
                worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                    "https://issuer.example",
                    "product-example",
                )
                .expect("the example external identity is valid"),
                admission::authenticated_principal::WorthQueryAuthenticationAudience::new("host")
                    .expect("the example audience is valid"),
                admission::authenticated_principal::WorthQueryAuthenticationMethod::new("example")
                    .expect("the example method is valid"),
                now,
                now + Duration::from_secs(3_600),
                Vec::new(),
            )
            .map_err(|_| {
                admission::authenticated_principal::WorthQueryAuthenticationAdapterFailure::new(
                    admission::authenticated_principal::WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

pub fn request_scope() -> admission::authenticated_principal::WorthQueryRequestScope {
    let cancellation = admission::authenticated_principal::WorthQueryCancellationSource::new();
    admission::authenticated_principal::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    )
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
