use super::*;
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
    time::{Duration, SystemTime},
};
use worth_query_host::facade::{
    admission::authenticated_principal as auth, declaration::authentication::*,
};
pub type App = installation::WorthQueryProgramApplicationRuntime<Schema, Program>;
pub fn identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://dispatch.invalid", "operator").unwrap()
}
pub fn install(width: u64, selections: Selections) -> App {
    install_with_cell(width, selections, false)
}
pub fn install_with_cell(width: u64, selections: Selections, self_supersede: bool) -> App {
    install_source(width, selections, self_supersede, None)
}
pub fn restore(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
    selections: Selections,
) -> App {
    install_source(1, selections, false, Some(checkpoint))
}
fn install_source(
    width: u64,
    selections: Selections,
    self_supersede: bool,
    checkpoint: Option<installation::WorthQueryApplicationCheckpoint>,
) -> App {
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let limits = installation::WorthQueryApplicationLimits::new(
        worth_query_testkit_document_retention::facade::document_world_resources::world_resources(
            256,
            4 * 1024 * 1024,
            256,
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).unwrap(),
        ),
        SignalConditionalEvaluationBudget::development(),
    );
    if let Some(checkpoint) = checkpoint {
        return installation::WorthQueryInterimCheckpointRestore::program(
            program,
            schema,
            (Configuration {
                selections,
                self_supersede,
            },),
            installation::ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111")
                .unwrap(),
            limits,
            checkpoint,
        )
        .unwrap();
    }
    installation::in_memory_program(
        program,
        schema,
        (Configuration {
            selections,
            self_supersede,
        },),
        installation::ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111")
            .unwrap(),
        limits,
        |_, graph, installed| {
            graph.bind_principal(
                &installed
                    .principal_binding(PrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Node::reference(),
                    WorthQueryApplicationEntityKey::new("root").unwrap(),
                )
                .field(Key::reference(), "root".to_string())
                .field(ParentName::reference(), String::new())
                .field(Value::reference(), 1)
                .field(Width::reference(), width)
                .field(Produced::reference(), 0),
            )?;
            if self_supersede {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        producer_cell::ProducerCell::reference(),
                        WorthQueryApplicationEntityKey::new("cell:root.0").unwrap(),
                    )
                    .field(producer_cell::CellKey::reference(), "root.0".to_string()),
                )?;
            }
            Ok(())
        },
    )
    .unwrap()
}
pub struct Local;
impl auth::WorthQueryAuthenticationAdapter for Local {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "dispatch-local"
    }
    fn validate<'a>(
        &'a self,
        _: (),
        _: &'a auth::WorthQueryRequestScope,
    ) -> auth::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async {
            let now = SystemTime::now();
            auth::WorthQueryValidatedExternalPrincipal::new(
                identity(),
                auth::WorthQueryAuthenticationAudience::new("dispatch").unwrap(),
                auth::WorthQueryAuthenticationMethod::new("local").unwrap(),
                now,
                now + Duration::from_secs(600),
                Vec::new(),
            )
            .map_err(|_| {
                auth::WorthQueryAuthenticationAdapterFailure::new(
                    auth::WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}
pub fn authenticate(
    app: &App,
) -> (
    auth::WorthQueryRequestScope,
    auth::WorthQueryAuthenticatedExternalPrincipal<Schema>,
) {
    let (scope, principal, _) = authenticate_cancellable(app);
    (scope, principal)
}
pub fn authenticate_cancellable(
    app: &App,
) -> (
    auth::WorthQueryRequestScope,
    auth::WorthQueryAuthenticatedExternalPrincipal<Schema>,
    auth::WorthQueryCancellationSource,
) {
    let cancel = auth::WorthQueryCancellationSource::new();
    let scope = auth::WorthQueryRequestScope::new(None, cancel.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let adapter = auth::admit_authentication_adapter(
        app.installed_schema(),
        auth::WorthQueryAuthenticationAdapterAdmission::new(
            auth::WorthQueryAuthenticationAudience::new("dispatch").unwrap(),
            auth::WorthQueryAuthenticationMethod::new("local").unwrap(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        Local,
    )
    .unwrap();
    let principal = {
        let mut future = pin!(adapter.authenticate((), &scope));
        match future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            Poll::Ready(result) => result.unwrap(),
            Poll::Pending => panic!("local authentication is synchronous"),
        }
    };
    (scope, principal, cancel)
}
