//! Application startup through public presets and local-process authentication.
mod bounded_history_close;
mod limits_presets;
mod local_principal;

use std::sync::{atomic::AtomicUsize, Arc};
use std::time::Duration;
use worth_query_decl::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial},
    application_installation::{ApplicationBuildIdentity, WorthQueryApplicationLimits},
    primary_graph::{WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphInstallationDenial},
    WorthQueryApplicationHome, WorthQueryApplicationHomeOpenRefused,
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};

worth_query_decl::facade::worth_query_authentication_audience!(HostAudience);
worth_query_decl::facade::worth_query_principal_issuer!(IssuerExampleIssuer);
worth_query_decl::facade::worth_query_authentication_method!(TestMethod);

pub(super) type Schema = fixture::schema::HomeIdentitySchema;
pub(super) type Home = WorthQueryApplicationHome<Schema, fixture::HomeProgram>;

fn issuer(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(IssuerExampleIssuer, subject).unwrap()
}

/// Opens a fresh Directory Home whose seed maps `subject` (enabled) and
/// `disabled` (disabled); every other identity is unmapped.
pub(super) fn open_home(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limits: WorthQueryApplicationLimits,
) -> Result<Home, WorthQueryApplicationHomeOpenRefused<Schema, fixture::HomeProgram>> {
    open_home_with_contacts(execution, limits, Arc::new(AtomicUsize::new(0)))
}

/// Same as [`open_home`] while exposing the handler contact counter.
pub(super) fn open_home_with_contacts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limits: WorthQueryApplicationLimits,
    contacts: Arc<AtomicUsize>,
) -> Result<Home, WorthQueryApplicationHomeOpenRefused<Schema, fixture::HomeProgram>> {
    let request = fixture::request_with_limits(
        fixture::location(),
        ApplicationBuildIdentity::parse("33333333-3333-4333-8333-333333333333").unwrap(),
        contacts,
        limits,
        |_, graph, schema| {
            fixture::schema::seed(graph, schema)?;
            graph.bind_principal(
                &schema
                    .principal_binding(fixture::schema::IdentityBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("disabled-principal").unwrap(),
                2_u64,
                issuer("disabled"),
                WorthQueryPrincipalMappingStatus::Disabled,
            )?;
            Ok::<(), WorthQueryPrimaryGraphInstallationDenial>(())
        },
    );
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .expect("the bootstrap execution opens")
}

pub(super) fn close_home(home: Home, execution: &worth_execution::ExecutionRequest<'_, '_>) {
    let closed = home
        .close(execution)
        .unwrap_or_else(|refused| panic!("an idle Home closes: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();
}

pub(super) fn scope(clock: &WorthQueryAdmissionClock) -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    )
}

/// The installation's admitted local principal for `subject`.
pub(super) fn local_principal(
    home: &Home,
    clock: &WorthQueryAdmissionClock,
    subject: &str,
    lifetime: Duration,
) -> WorthQueryAdmittedAuthenticationAdapter<Schema, WorthQueryLocalProcessPrincipal> {
    admit_authentication_adapter(
        home.runtime().runtime().installed_schema(),
        admission(clock),
        WorthQueryLocalProcessPrincipal::new(issuer(subject), HostAudience, TestMethod, lifetime)
            .unwrap(),
    )
    .unwrap()
}

fn admission(clock: &WorthQueryAdmissionClock) -> WorthQueryAuthenticationAdapterAdmission {
    WorthQueryAuthenticationAdapterAdmission::new(
        WorthQueryAuthenticationAudience::of(HostAudience),
        WorthQueryAuthenticationMethod::of(TestMethod),
        clock.clone(),
    )
}

/// The seeded balance read, or the request's refusal.
pub(super) fn read_balance(
    home: &Home,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &WorthQueryRequestScope,
) -> Result<Vec<u64>, String> {
    home.runtime()
        .request(principal, scope)
        .query(fixture::balance_query::AccountRead(
            "account-001".to_owned(),
        ))
        .execute()
        .map(|result| result.rows().to_vec())
        .map_err(|denial| match denial {
            WorthQueryApplicationRequestQueryDenial::PrincipalResolution(refusal) => {
                format!("principal resolution {:?}", refusal.kind())
            }
            other => format!("{other:?}"),
        })
}
