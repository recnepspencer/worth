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
    install_source(width, selections, Fault::None, 1, None)
}
pub fn install_two_levels() -> App {
    install_source(2, Arc::default(), Fault::None, 0, None)
}
pub fn install_fault(fault: Fault) -> App {
    install_source(2, Arc::default(), fault, 1, None)
}
pub fn restore(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
    selections: Selections,
) -> App {
    install_source(1, selections, Fault::None, 1, Some(checkpoint))
}
pub fn try_restore(
    checkpoint: installation::WorthQueryApplicationCheckpoint,
) -> Result<App, installation::WorthQueryInMemoryApplicationDenial> {
    try_install_source(1, Arc::default(), Fault::None, 1, Some(checkpoint))
}
fn install_source(
    width: u64,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    checkpoint: Option<installation::WorthQueryApplicationCheckpoint>,
) -> App {
    try_install_source(width, selections, fault, child_width, checkpoint).unwrap()
}
fn try_install_source(
    width: u64,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    checkpoint: Option<installation::WorthQueryApplicationCheckpoint>,
) -> Result<App, installation::WorthQueryInMemoryApplicationDenial> {
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let limits = installation::WorthQueryInMemoryApplicationLimits::new(
        worth_query_testkit_document_retention::facade::document_world_resources::world_resources(
            256,
            4 * 1024 * 1024,
            256,
        ),
        worth_query_host::facade::runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).unwrap(),
        ),
        SignalConditionalEvaluationBudget::development(),
    );
    if let Some(checkpoint) = checkpoint {
        return installation::in_memory_program_from_checkpoint(
            program,
            schema,
            (Configuration {
                selections,
                fault,
                child_width,
            },),
            installation::ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111")
                .unwrap(),
            limits,
            checkpoint,
        );
    }
    installation::in_memory_program(
        program,
        schema,
        (Configuration {
            selections,
            fault,
            child_width,
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
            if fault == Fault::Extra {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        realization::Realization::reference(),
                        WorthQueryApplicationEntityKey::new("unrelated").unwrap(),
                    )
                    .field(
                        realization::RealizationKey::reference(),
                        "unrelated".to_owned(),
                    )
                    .field(realization::RealizedValue::reference(), 1),
                )?;
            }
            Ok(())
        },
    )
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
    let scope = auth::WorthQueryRequestScope::new(None, cancel.token());
    let adapter = auth::admit_authentication_adapter(
        app.installed_schema(),
        auth::WorthQueryAuthenticationAdapterAdmission::new(
            auth::WorthQueryAuthenticationAudience::new("dispatch").unwrap(),
            auth::WorthQueryAuthenticationMethod::new("local").unwrap(),
        ),
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
