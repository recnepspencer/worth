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
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::directory_home::{
    with_directory_bootstrap, DirectoryHomeGuard,
};

worth_query_decl::facade::worth_query_authentication_audience!(DispatchAudience);
worth_query_decl::facade::worth_query_principal_issuer!(DispatchInvalidIssuer);
worth_query_decl::facade::worth_query_authentication_method!(LocalMethod);
pub type App = installation::WorthQueryProgramApplicationRuntime<Schema, Program>;
pub fn identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(DispatchInvalidIssuer, "operator").unwrap()
}
pub fn install(width: u64, selections: Selections) -> App {
    install_with_cell(width, selections, false)
}
pub fn install_with_cell(width: u64, selections: Selections, self_supersede: bool) -> App {
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    installation::in_memory_program(
        program,
        schema,
        (Configuration {
            selections,
            self_supersede,
        },),
        installation::ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111")
            .unwrap(),
        limits(),
        move |_, graph, installed| seed_graph(width, self_supersede, graph, installed),
    )
    .unwrap()
}
pub fn open_directory(
    directory: PathBuf,
    width: u64,
    selections: Selections,
    self_supersede: bool,
) -> DirectoryHomeGuard<Schema, Program> {
    let schema = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let home = with_directory_bootstrap(|phase| {
        WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.clone()),
            program,
            installation::WorthQueryApplicationProgramRoster::new(),
            schema,
            (Configuration {
                selections,
                self_supersede,
            },),
            installation::ApplicationBuildIdentity::parse("11111111-1111-4111-8111-111111111111")
                .unwrap(),
            limits(),
            move |_, graph, installed| seed_graph(width, self_supersede, graph, installed),
        )
        .open_in_advancement(phase)
    })
    .unwrap();
    DirectoryHomeGuard::wrap(home, directory)
}
fn limits() -> installation::WorthQueryApplicationLimits {
    installation::WorthQueryApplicationLimits::new(
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
    )
}
fn seed_graph(
    width: u64,
    self_supersede: bool,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
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
    if self_supersede {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                producer_cell::ProducerCell::reference(),
                WorthQueryApplicationEntityKey::new("cell:root.0").unwrap(),
            )
            .field(producer_cell::CellKey::at("CellKey"), "root.0".to_string()),
        )?;
    }
    Ok(())
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
            auth::WorthQueryAuthenticationAudience::of(DispatchAudience),
            auth::WorthQueryAuthenticationMethod::of(LocalMethod), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
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
