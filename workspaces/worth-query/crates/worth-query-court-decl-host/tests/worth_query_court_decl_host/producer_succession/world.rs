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
) -> Result<App, installation::WorthQueryApplicationPreparationDenial> {
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
) -> Result<App, installation::WorthQueryApplicationPreparationDenial> {
    try_install_program::<Program>(width, selections, fault, child_width, false, checkpoint)
}
pub(super) fn try_install_program<
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
>(
    width: u64,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    read_predecessor_orbits: bool,
    checkpoint: Option<installation::WorthQueryApplicationCheckpoint>,
) -> Result<
    installation::WorthQueryProgramApplicationRuntime<Schema, P>,
    installation::WorthQueryApplicationPreparationDenial,
>
where
    P::Outputs: installation::WorthQueryApplicationProgramRoots<Schema>,
{
    try_install_program_with_sources::<P>(
        width,
        selections,
        fault,
        child_width,
        read_predecessor_orbits,
        checkpoint,
        &[],
    )
}
pub fn install_degree() -> App {
    try_install_program_with_sources::<Program>(
        0,
        Arc::default(),
        Fault::None,
        0,
        false,
        None,
        &["outside"],
    )
    .unwrap()
}
fn try_install_program_with_sources<
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
>(
    width: u64,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    read_predecessor_orbits: bool,
    checkpoint: Option<installation::WorthQueryApplicationCheckpoint>,
    others: &[&str],
) -> Result<
    installation::WorthQueryProgramApplicationRuntime<Schema, P>,
    installation::WorthQueryApplicationPreparationDenial,
>
where
    P::Outputs: installation::WorthQueryApplicationProgramRoots<Schema>,
{
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, P>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let limits = installation::WorthQueryApplicationLimits::new(
        worth_query_testkit_document_retention::facade::document_world_resources::world_resources(
            256,
            4 * 1024 * 1024,
            256,
            auth::WorthQueryAdmissionClock::system(),
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
                fault,
                child_width,
                read_predecessor_orbits,
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
            read_predecessor_orbits,
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
                .field(Key::at("Key"), "root".to_string())
                .field(ParentName::at("ParentName"), String::new())
                .field(Value::at("Value"), 1)
                .field(Width::at("Width"), width)
                .field(Produced::at("Produced"), 0),
            )?;
            for key in others {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Node::reference(),
                        WorthQueryApplicationEntityKey::new(*key).unwrap(),
                    )
                    .field(Key::at("Key"), (*key).to_owned())
                    .field(ParentName::at("ParentName"), String::new())
                    .field(Value::at("Value"), 1)
                    .field(Width::at("Width"), 0)
                    .field(Produced::at("Produced"), 0),
                )?;
            }
            if super::PRIVATE_AGGREGATES {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Node::reference(),
                        WorthQueryApplicationEntityKey::new("observer").unwrap(),
                    )
                    .field(Key::at("Key"), "observer".to_owned())
                    .field(ParentName::at("ParentName"), String::new())
                    .field(Value::at("Value"), 8)
                    .field(Width::at("Width"), 0)
                    .field(Produced::at("Produced"), 0),
                )?;
            }
            if matches!(fault, Fault::Extra | Fault::HiddenDeletion) {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        realization::Realization::reference(),
                        WorthQueryApplicationEntityKey::new("unrelated").unwrap(),
                    )
                    .field(
                        realization::RealizationKey::at("RealizationKey"),
                        "unrelated".to_owned(),
                    )
                    .field(realization::RealizedValue::at("RealizedValue"), 1),
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
pub fn authenticate<P: ApplicationProgramDefinition<Schema>>(
    app: &installation::WorthQueryProgramApplicationRuntime<Schema, P>,
) -> (
    auth::WorthQueryRequestScope,
    auth::WorthQueryAuthenticatedExternalPrincipal<Schema>,
) {
    let (scope, principal, _) = authenticate_cancellable(app);
    (scope, principal)
}
pub fn authenticate_cancellable<P: ApplicationProgramDefinition<Schema>>(
    app: &installation::WorthQueryProgramApplicationRuntime<Schema, P>,
) -> (
    auth::WorthQueryRequestScope,
    auth::WorthQueryAuthenticatedExternalPrincipal<Schema>,
    auth::WorthQueryCancellationSource,
) {
    let cancel = auth::WorthQueryCancellationSource::new();
    let scope = auth::WorthQueryRequestScope::new(
        None,
        cancel.token(),
        auth::WorthQueryAdmissionClock::system(),
    );
    let adapter = auth::admit_authentication_adapter(
        app.installed_schema(),
        auth::WorthQueryAuthenticationAdapterAdmission::new(
            auth::WorthQueryAuthenticationAudience::new("dispatch").unwrap(),
            auth::WorthQueryAuthenticationMethod::new("local").unwrap(),
            scope.clock().clone(),
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
