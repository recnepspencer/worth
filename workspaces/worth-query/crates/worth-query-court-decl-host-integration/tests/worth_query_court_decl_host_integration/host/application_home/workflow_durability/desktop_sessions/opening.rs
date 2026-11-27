//! A workflow Home opened through `WorthQueryHomeConfig` under a held trusted
//! clock, and the principals its sessions act under.
use super::*;
use document::programs::WorkflowOwnerProgram;
use document::schema::DocumentRetentionSchema;
use std::time::{SystemTime, UNIX_EPOCH};
use worth_foundational::facade::CanonicalTimestamp;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryAuthenticatedExternalPrincipal,
        WorthQueryLocalProcessPrincipal,
    },
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::WorthQueryFixedRuntimeTimeSource,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
};

// The installed principal binding admits this audience and method by name.
worth_query_decl::facade::worth_query_authentication_audience!(DocumentRetentionHostAudience);
worth_query_decl::facade::worth_query_authentication_method!(CertificationMethod);

/// Trusted time held at the current instant, so the fixture's real proofs
/// (valid for an hour from now) stay inside every admission window.
pub(super) fn trusted_now() -> WorthQueryFixedRuntimeTimeSource {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    WorthQueryFixedRuntimeTimeSource::new(CanonicalTimestamp {
        micros_since_unix_epoch: i64::try_from(since_epoch.as_micros()).unwrap(),
    })
}

/// Opens (or reopens) the Home at `location` through the desktop
/// configuration, with the fixture's rostered programs, then binds a freshly
/// installed workflow vocabulary.
pub(super) fn open(
    location: &Location,
    trusted: &WorthQueryFixedRuntimeTimeSource,
) -> (Home, CertificationAuthenticationOwner) {
    let config = WorthQueryHomeConfig::<DocumentRetentionSchema, WorkflowOwnerProgram>::new(
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        ((),),
    )
    .with_limits(document::host::host_limits(
        WorthQueryAdmissionClock::system(),
    ))
    .with_roster(
        WorthQueryApplicationProgramRoster::new()
            .support(document::programs::validate::<
                super::super::target_program::TargetProgram,
            >())
            .support(document::actor_generation::validated_actor_program())
            .support(document::model_lifecycle::validated_model_lifecycle_program()),
    )
    .with_authorization_time_source(trusted.clone())
    .with_initial_state(document::host::seed_host);
    let home = match location {
        Location::MemoryOnly => WorthQueryApplicationHome::open_memory(config),
        Location::Directory(path) => WorthQueryApplicationHome::open_directory(path, config),
    }
    .unwrap_or_else(|denial| panic!("the workflow Home opens: {denial}"));
    let workflow = install(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
    );
    let authentication = install_certification_authentication(home.runtime().installed_schema());
    let home = home
        .retain_workflow_spec(workflow, authentication.signing_owner())
        .unwrap();
    (home, authentication)
}

/// The operator acting as this operating-system process.
pub(super) fn local_operator() -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        operator_identity(),
        DocumentRetentionHostAudience,
        CertificationMethod,
        Duration::from_secs(600),
    )
    .unwrap()
}

/// The two externally authenticated principals the sessions bind.
pub(super) fn principals(
    home: &Home,
) -> (
    WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
) {
    let schema = home.runtime().installed_schema();
    let scope = request_scope();
    (
        authenticate_operator(schema, &scope),
        authenticate_reviewer(schema, &scope),
    )
}
