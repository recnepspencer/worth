use super::*;
use std::{
    future::Future,
    path::PathBuf,
    pin::pin,
    task::{Context, Poll, Waker},
    time::{Duration, SystemTime},
};
use worth_query_host::facade::{
    admission::authenticated_principal as auth, declaration::authentication::*,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeOpenRefused,
    WorthQueryApplicationHomeRequest,
};

worth_query_host::facade::worth_query_authentication_audience!(DispatchAudience);
worth_query_host::facade::worth_query_principal_issuer!(DispatchInvalidIssuer);
worth_query_host::facade::worth_query_authentication_method!(LocalMethod);
use worth_query_testkit_document_retention::facade::directory_home::{
    unique_directory, DirectoryHomeGuard,
};
pub type App = DirectoryHomeGuard<Schema, Program>;
pub type RuntimeApp = installation::WorthQueryProgramApplicationRuntime<Schema, Program>;
pub fn identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(DispatchInvalidIssuer, "operator").unwrap()
}
pub fn install(width: u64, selections: Selections) -> App {
    install_source(width, selections, Fault::None, 1)
}
pub fn install_memory(width: u64, selections: Selections) -> RuntimeApp {
    open_memory_program_with_sources::<Program>(width, selections, Fault::None, 1, false, &[])
}
pub fn install_two_levels() -> RuntimeApp {
    open_memory_program_with_sources::<Program>(2, Arc::default(), Fault::None, 0, false, &[])
}
pub fn install_fault(fault: Fault) -> RuntimeApp {
    open_memory_program_with_sources::<Program>(2, Arc::default(), fault, 1, false, &[])
}
pub fn native_head(app: &RuntimeApp) -> (u64, Vec<u64>) {
    app.with_host_advancement(|phase| {
        let retained = worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation::retain_on_branch(
            &phase,
            app.runtime(),
            app.current_world(),
        )
        .unwrap();
        let selected = retained.select_on(&phase, app.runtime()).unwrap();
        let target = selected
            .product()
            .relational_basis_descriptor()
            .reference()
            .target()
            .as_basis()
            .unwrap();
        (
            target.selected_commit_id(),
            target.parent_commit_ids().to_vec(),
        )
    })
    .unwrap()
}
pub fn reopen(directory: PathBuf, selections: Selections) -> App {
    try_open_program_with_sources::<Program>(
        directory,
        1,
        selections,
        Fault::None,
        1,
        false,
        &[],
    )
    .unwrap_or_else(|refused| panic!("producer_succession directory reopen refused: {refused:?}"))
}
pub fn try_reopen(
    directory: PathBuf,
) -> Result<App, WorthQueryApplicationHomeOpenRefused<Schema, Program>> {
    try_open_program_with_sources::<Program>(
        directory,
        1,
        Arc::default(),
        Fault::None,
        1,
        false,
        &[],
    )
}
fn install_source(width: u64, selections: Selections, fault: Fault, child_width: u64) -> App {
    try_install_program::<Program>(width, selections, fault, child_width, false)
        .unwrap_or_else(|refused| panic!("producer_succession directory open refused: {refused:?}"))
}
pub(super) fn try_install_program<
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
>(
    width: u64,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    read_predecessor_orbits: bool,
) -> Result<DirectoryHomeGuard<Schema, P>, WorthQueryApplicationHomeOpenRefused<Schema, P>>
where
    P::Outputs: installation::WorthQueryApplicationProgramRoots<Schema>,
{
    try_open_program_with_sources::<P>(
        unique_directory("producer-succession"),
        width,
        selections,
        fault,
        child_width,
        read_predecessor_orbits,
        &[],
    )
}
pub fn install_degree() -> RuntimeApp {
    open_memory_program_with_sources::<Program>(
        0,
        Arc::default(),
        Fault::None,
        0,
        false,
        &["outside"],
    )
}
fn open_memory_program_with_sources<
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
>(
    width: u64,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    read_predecessor_orbits: bool,
    others: &[&str],
) -> installation::WorthQueryProgramApplicationRuntime<Schema, P>
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
            seed_entities(graph, width, fault, others)
        },
    )
    .unwrap_or_else(|refused| panic!("producer_succession memory open refused: {refused:?}"))
}
fn seed_entities(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    width: u64,
    fault: Fault,
    others: &[&str],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
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
}
fn try_open_program_with_sources<
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
>(
    directory: PathBuf,
    width: u64,
    selections: Selections,
    fault: Fault,
    child_width: u64,
    read_predecessor_orbits: bool,
    others: &[&str],
) -> Result<DirectoryHomeGuard<Schema, P>, WorthQueryApplicationHomeOpenRefused<Schema, P>>
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
    let home = worth_query_testkit_document_retention::facade::directory_home::with_directory_bootstrap(
        |phase| {
            WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(directory.clone()),
                program,
                installation::WorthQueryApplicationProgramRoster::new(),
                schema,
                (Configuration {
                    selections,
                    fault,
                    child_width,
                    read_predecessor_orbits,
                },),
                installation::ApplicationBuildIdentity::parse(
                    "11111111-1111-4111-8111-111111111111",
                )
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
                    seed_entities(graph, width, fault, others)
                },
            )
            .open_in_advancement(phase)
        },
    )?;
    Ok(DirectoryHomeGuard::wrap(home, directory))
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
                auth::WorthQueryAuthenticationAudience::of(DispatchAudience),
                auth::WorthQueryAuthenticationMethod::of(LocalMethod),
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
            auth::WorthQueryAuthenticationAudience::of(DispatchAudience),
            auth::WorthQueryAuthenticationMethod::of(LocalMethod),
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
