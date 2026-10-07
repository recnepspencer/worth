//! Fresh authentic application worlds and credentials, separate from the reference.
use super::{application::*, expected_history::World};
use crate::principal::*;
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
    time::{Duration, Instant, SystemTime},
};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    admission::authenticated_principal as authentication,
    application_installation,
    declaration::authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
    },
    primary_graph::*,
};
pub(super) type Application =
    application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>;
pub(super) fn install(world: &World) -> Application {
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .validated_program()
        .unwrap();
    let declaration = Schema::declaration().unwrap();
    let limits = super::super::support::limits(
        32,
        super::super::support::invalidation(128 * 1024 * 1024, 1_000_000, 32),
    );
    application_installation::in_memory_program(
        program,
        declaration,
        ((),),
        limits,
        |graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
                .unwrap();
            graph.bind_principal(
                &principal,
                WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1,
                external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for member in &world.members {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Node::reference::<Schema>(),
                        WorthQueryApplicationEntityKey::new(format!("member-{}", member.key))
                            .unwrap(),
                    )
                    .field(NodeKey::reference(), member.key)
                    .field(Value::reference(), 0)
                    .field(Applied::reference(), 0),
                )?;
            }
            Ok(())
        },
    )
    .unwrap()
}
// The existing support adapter is fixed to CheckpointSchema and its
// installed principal binding. This independent Node schema needs its own
// admitted adapter; changing the existing test owner would exceed mount scope.
struct LocalIdentityAdapter;
struct LocalCredential;

impl authentication::WorthQueryAuthenticationAdapter for LocalIdentityAdapter {
    type Credential = LocalCredential;

    fn configuration_identity(&self) -> &str {
        "worth.query.certification.neutral-history-authentication.v1"
    }

    fn validate<'a>(
        &'a self,
        _: Self::Credential,
        _: &'a authentication::WorthQueryRequestScope,
    ) -> authentication::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            authentication::WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                authentication::WorthQueryAuthenticationAudience::new("neutral-history").unwrap(),
                authentication::WorthQueryAuthenticationMethod::new("local").unwrap(),
                now,
                now + Duration::from_secs(600),
                Vec::new(),
            )
            .map_err(|_| {
                authentication::WorthQueryAuthenticationAdapterFailure::new(
                    authentication::WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://neutral-history.invalid/local", "model-owner")
        .unwrap()
}

pub(super) fn authenticate<Program>(
    application: &application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>,
) -> (
    authentication::WorthQueryRequestScope,
    authentication::WorthQueryAuthenticatedExternalPrincipal<Schema>,
)
where
    Program: ApplicationProgramDefinition<Schema>,
{
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let scope = authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    );
    let adapter = authentication::admit_authentication_adapter(
        application.installed_schema(),
        authentication::WorthQueryAuthenticationAdapterAdmission::new(
            authentication::WorthQueryAuthenticationAudience::new("neutral-history").unwrap(),
            authentication::WorthQueryAuthenticationMethod::new("local").unwrap(),
        ),
        LocalIdentityAdapter,
    )
    .unwrap();
    let principal = block_on(adapter.authenticate(LocalCredential, &scope)).unwrap();
    (scope, principal)
}

fn block_on<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("the local authentication completes synchronously"),
    }
}
