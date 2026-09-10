use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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

#[path = "adapters/clock.rs"]
mod clock;
pub use clock::{ClockController, ClockSource, CourtroomClock};
#[path = "adapters/predicate.rs"]
mod predicate;
pub use predicate::{Predicate, ReplacementPredicate};
#[path = "adapters/transport.rs"]
mod transport;
pub use transport::CompletingExternalTransport;

#[derive(Clone)]
pub struct PanicController {
    panic: Arc<AtomicBool>,
}

impl PanicController {
    pub fn set(&self, panic: bool) {
        self.panic.store(panic, Ordering::SeqCst);
    }
}

pub struct IntentProjector;

impl
    domain::WorthQueryTemporalIntentProjector<
        TemporalReadyNode,
        CourtroomClock,
        IntentQueryResult,
        TemporalInput,
    > for IntentProjector
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.courtroom.projector";

    fn project(
        &self,
        row: &IntentQueryResult,
    ) -> Result<
        domain::WorthQueryTemporalIntentCandidate<CourtroomClock, TemporalInput>,
        domain::WorthQueryTemporalIntentProjectionFailure,
    > {
        let identity = domain::WorthQueryTemporalIntentIdentity::declare(row.identity.clone())
            .map_err(|detail| projection_failure(detail))?;
        let input_identity =
            domain::WorthQueryTemporalOperationInputIdentity::declare(row.input.clone())
                .map_err(|detail| projection_failure(detail))?;
        let idempotency = domain::WorthQueryTemporalIntentIdempotencyRelation::declare(format!(
            "{}:{}:{}",
            row.identity, row.revision, row.input
        ))
        .map_err(|detail| projection_failure(detail))?;
        let due = domain::WorthQueryClockCoordinate::from_nanoseconds(row.due);
        let input = TemporalInput(row.input.clone());
        Ok(match row.lifecycle.as_str() {
            "active" => domain::WorthQueryTemporalIntentCandidate::active(
                identity,
                row.identity.clone(),
                row.revision,
                due,
                input,
                input_identity,
                idempotency,
            ),
            "cancelled" => domain::WorthQueryTemporalIntentCandidate::cancelled(
                identity,
                row.identity.clone(),
                row.revision,
                due,
                input,
                input_identity,
                idempotency,
            ),
            _ => domain::WorthQueryTemporalIntentCandidate::completed(
                identity,
                row.identity.clone(),
                row.revision,
                due,
                input,
                input_identity,
                idempotency,
            ),
        })
    }
}

fn projection_failure(detail: &'static str) -> domain::WorthQueryTemporalIntentProjectionFailure {
    domain::WorthQueryTemporalIntentProjectionFailure::new(
        domain::WorthQueryTemporalIntentProjectionFailureKind::InvalidIdentity,
        detail,
    )
}

pub struct Invoker {
    panic: PanicController,
    contacts: ContactCounters,
}

impl Invoker {
    pub fn controlled(contacts: ContactCounters) -> (Self, PanicController) {
        let controller = PanicController {
            panic: Arc::new(AtomicBool::new(false)),
        };
        (
            Self {
                panic: controller.clone(),
                contacts,
            },
            controller,
        )
    }
}

impl
    primary_graph::WorthQueryTemporalOperationInvoker<
        TemporalHostSchema,
        ExecuteTemporal,
        TemporalInput,
        TemporalIntent,
    > for Invoker
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.courtroom.invoker";
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
        self.contacts.preconditions.fetch_add(1, Ordering::SeqCst);
        assert!(
            !self.panic.panic.load(Ordering::SeqCst),
            "preconditions panic"
        );
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
        self.contacts.projection.fetch_add(1, Ordering::SeqCst);
        reader
            .require_decision_field(scope, IntentIdentityField::reference())
            .map_err(|denial| {
                primary_graph::WorthQueryTemporalInvocationFailure::new(
                    primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
                    denial.to_string(),
                )
            })?;
        let identity = reader
            .decision_field(scope, IntentIdentityField::reference())
            .map_err(|denial| {
                primary_graph::WorthQueryTemporalInvocationFailure::new(
                    primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
                    denial.to_string(),
                )
            })?
            .ok_or_else(|| {
                primary_graph::WorthQueryTemporalInvocationFailure::new(
                    primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
                    "required temporal identity was absent",
                )
            })?;
        reader
            .decision_field(scope, IntentEffectField::reference())
            .map_err(|denial| {
                primary_graph::WorthQueryTemporalInvocationFailure::new(
                    primary_graph::WorthQueryTemporalInvocationFailureKind::ProjectionRejected,
                    denial.to_string(),
                )
            })?;
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
        self.contacts.apply.fetch_add(1, Ordering::SeqCst);
        let target = effects.projected_entity(&target).map_err(|denial| {
            primary_graph::WorthQueryTemporalInvocationFailure::new(
                primary_graph::WorthQueryTemporalInvocationFailureKind::InvocationRejected,
                denial.to_string(),
            )
        })?;
        effects
            .write_field(&target, IntentEffectField::reference(), input.0)
            .map_err(|denial| {
                primary_graph::WorthQueryTemporalInvocationFailure::new(
                    primary_graph::WorthQueryTemporalInvocationFailureKind::InvocationRejected,
                    denial.to_string(),
                )
            })?;
        effects
            .emit(
                TemporalExecutionEffect::reference(),
                TemporalExecutionNotice { identity },
            )
            .map_err(|denial| {
                primary_graph::WorthQueryTemporalInvocationFailure::new(
                    primary_graph::WorthQueryTemporalInvocationFailureKind::InvocationRejected,
                    denial.to_string(),
                )
            })?;
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct ContactCounters {
    predicate: Arc<AtomicUsize>,
    preconditions: Arc<AtomicUsize>,
    projection: Arc<AtomicUsize>,
    apply: Arc<AtomicUsize>,
}

impl ContactCounters {
    pub fn snapshot(&self) -> (usize, usize, usize, usize) {
        (
            self.predicate.load(Ordering::SeqCst),
            self.preconditions.load(Ordering::SeqCst),
            self.projection.load(Ordering::SeqCst),
            self.apply.load(Ordering::SeqCst),
        )
    }
}

pub struct PrincipalSource {
    adapter: Arc<
        admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
            TemporalHostSchema,
            IdentityAdapter,
        >,
    >,
    panic: PanicController,
}

impl PrincipalSource {
    pub fn controlled(
        adapter: admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
            TemporalHostSchema,
            IdentityAdapter,
        >,
    ) -> (Self, PanicController) {
        let panic = PanicController {
            panic: Arc::new(AtomicBool::new(false)),
        };
        (
            Self {
                adapter: Arc::new(adapter),
                panic: panic.clone(),
            },
            panic,
        )
    }
}

impl primary_graph::WorthQueryTemporalPrincipalSource<TemporalHostSchema> for PrincipalSource {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.host.courtroom.principal-source";

    fn admit(
        &self,
    ) -> Result<
        primary_graph::WorthQueryTemporalPrincipalAdmission<TemporalHostSchema>,
        primary_graph::WorthQueryTemporalPrincipalFailure,
    > {
        assert!(!self.panic.panic.load(Ordering::SeqCst), "principal panic");
        let cancellation = admission::authenticated_principal::WorthQueryCancellationSource::new();
        let scope = admission::authenticated_principal::WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(60),
            cancellation.token(),
        );
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
        "worth-query-temporal-courtroom-adapter-v1"
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
                    "temporal-host",
                )
                .unwrap(),
                admission::authenticated_principal::WorthQueryAuthenticationAudience::new("host")
                    .unwrap(),
                admission::authenticated_principal::WorthQueryAuthenticationMethod::new("test")
                    .unwrap(),
                now,
                now + Duration::from_secs(3600),
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

pub(super) fn block_on<F: Future>(future: F) -> F::Output {
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
