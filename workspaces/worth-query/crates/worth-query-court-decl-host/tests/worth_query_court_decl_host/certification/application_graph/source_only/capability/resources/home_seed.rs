//! One real installed Directory journey for process-local release and cold custody.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{
    application_program::ApplicationProgramAuthoring,
    authentication::WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{ApplicationBuildIdentity, WorthQueryApplicationProgramRoster},
    primary_graph::*,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_certification_operation::facade::certification::source_only::capability::{
    host::{identity, seed_grant, Adapter, Contacts},
    program::Program,
    resources::{metadata, Resource},
    schema::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::host_limits,
    operator_identity::{block_on, request_scope},
};
type Home = WorthQueryApplicationHome<CapabilitySchema, Program>;

fn request(
    directory: std::path::PathBuf,
    contacts: Arc<Contacts>,
    initialized: Arc<AtomicUsize>,
) -> WorthQueryApplicationHomeRequest<'static, CapabilitySchema, Program> {
    let declaration = CapabilitySchema::declaration().unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(directory),
        ApplicationProgramAuthoring::<CapabilitySchema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (contacts,),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        host_limits(WorthQueryAdmissionClock::system()),
        move |_, graph, installed| {
            assert_eq!(
                initialized.fetch_add(1, Ordering::SeqCst),
                0,
                "reopen cannot replay bootstrap"
            );
            let binding = installed
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            for actor in [1, 2] {
                graph.bind_principal(
                    &binding,
                    WorthQueryApplicationPrincipalKey::new(format!("actor-{actor}")).unwrap(),
                    actor,
                    identity(actor),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
            }
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Document::reference(),
                    WorthQueryApplicationEntityKey::new("document-row-1").unwrap(),
                )
                .field(
                    DocumentIdentityField::at("DocumentIdentityField"),
                    DOCUMENT_IDENTITY.to_owned(),
                )
                .field(DocumentRetentionField::at("DocumentRetentionField"), 7_u64),
            )?;
            seed_grant(graph, "source-child", 1, 1)?;
            seed_grant(graph, "resource-action", 1, 3)?;
            seed_grant(graph, "issuer-revoke", 2, 2)?;
            Ok(())
        },
    )
}
fn authenticate(
    home: &Home,
    actor: u64,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema> {
    let adapter = admit_authentication_adapter(
        home.runtime().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("capability-retention-host").unwrap(),
            WorthQueryAuthenticationMethod::new("certification").unwrap(),
            WorthQueryAdmissionClock::system(),
        ),
        Adapter,
    )
    .unwrap();
    block_on(adapter.authenticate(actor, scope)).unwrap()
}
fn metadata_access(
    home: &Home,
    actor: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    scope: &WorthQueryRequestScope,
) -> metadata::Admitted {
    let application = home.runtime().runtime();
    application
        .with_application_advancement(scope, |phase| {
            let selected = application
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap();
            let installed = application.installed_schema();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &installed
                        .principal_binding(DocumentPrincipalBinding::reference())
                        .unwrap(),
                    actor,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            selected.admit_capability_access(
                &phase,
                &principal,
                &installed
                    .capability(
                        metadata::ResourceDisclosure::reference(),
                        metadata::InspectResource::reference(),
                    )
                    .unwrap(),
                metadata::MetadataInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                },
                scope,
            )
        })
        .unwrap()
        .unwrap()
}
#[test]
fn directory_resource_handoff_preserves_original_custody_and_actual_release() {
    let directory = std::env::temp_dir().join(format!(
        "worth-resource-custody-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(Contacts::default());
    let budget = worth_execution::SerialMemoryBudget::new(Some(256 << 20));
    let cancellation = worth_execution::CancellationSource::new();
    let host_request =
        worth_execution::SerialRequest::from_memory(budget, cancellation.token(), None);
    let execution = worth_execution::ExecutionRequest::serial(&host_request);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(directory.clone(), contacts.clone(), initialized.clone())
            .open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("resource Home: {cause:?}"));
    let original = {
        let scope = request_scope();
        let actor = authenticate(&home, 1, &scope);
        let application = home.runtime().runtime();
        let resources =
            application.application_resources(&actor, &scope, application.current_world());
        let handle = resources.start::<Resource>(41).unwrap_or_else(|stop| {
            let (_, cause) = stop.into_parts();
            panic!("real start: {cause:?}")
        });
        assert_eq!(contacts.resource.input(0), 41);
        contacts.resource.send(0, 123).unwrap();
        let identity = handle
            .obligation_identity()
            .expect("actual acknowledged original resource identity");
        assert!(
            resources.retained_state::<Resource>(&41, identity).is_err(),
            "action permission cannot supply metadata disclosure"
        );
        let state = resources
            .retained_state_governed::<Resource, _, _, _>(
                &41,
                identity,
                metadata_access(&home, &actor, &scope),
            )
            .unwrap();
        assert_eq!(state.identity, identity);
        assert!(!state.released);
        assert!(
            resources
                .discharge_retained::<Resource>(&41, state.identity)
                .is_err(),
            "fresh memory cannot release original live work"
        );
        drop(handle);
        state.identity
    };
    contacts
        .resource
        .cleanup_pending
        .store(true, Ordering::SeqCst);
    let refused = home
        .close(&execution)
        .err()
        .expect("actual provider Pending refuses close");
    assert!(
        matches!(
            refused.query_denial(),
            Some(WorthQueryHomeRetirementDenial::Resource(_))
        ),
        "{refused:?}"
    );
    let home = refused
        .resume()
        .unwrap_or_else(|cause| panic!("same resource owner: {cause:?}"));
    assert_eq!(contacts.resource.counts(), (1, 0, 1));
    assert!(
        contacts.resource.send(0, 999).is_err(),
        "handoff freezes ingress before discarding it"
    );
    contacts
        .resource
        .cleanup_pending
        .store(false, Ordering::SeqCst);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("actual original release: {cause:?}"))
        .reclaim();
    assert_eq!(contacts.resource.counts(), (1, 0, 2));
    assert!(contacts.resource.issued.lock().unwrap()[0].closed);
    let fresh = Arc::new(Contacts::default());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(directory, fresh.clone(), initialized.clone()).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("cold resource Home: {cause:?}"));
    {
        let identity = original;
        let scope = request_scope();
        let actor = authenticate(&home, 1, &scope);
        let application = home.runtime().runtime();
        let resources =
            application.application_resources(&actor, &scope, application.current_world());
        assert!(
            resources.retained_state::<Resource>(&41, identity).is_err(),
            "action permission cannot supply metadata disclosure"
        );
        let state = resources
            .retained_state_governed::<Resource, _, _, _>(
                &41,
                identity,
                metadata_access(&home, &actor, &scope),
            )
            .unwrap();
        assert_eq!(state.identity, original);
        assert!(state.released);
        assert_eq!(state.terminal, Some(WorthQueryResourceTerminal::Cancelled));
        assert_eq!(
            resources
                .with_retained_result_governed::<Resource, _, _, _, _>(
                    &41,
                    original,
                    metadata_access(&home, &actor, &scope),
                    |value| *value
                )
                .unwrap(),
            None,
            "unverified ingress cannot become a result after reopen"
        );
        resources
            .discharge_retained::<Resource>(&41, original)
            .unwrap();
        let absent = resources
            .retained_state_governed::<Resource, _, _, _>(
                &41,
                original,
                metadata_access(&home, &actor, &scope),
            )
            .unwrap_err();
        assert_eq!(absent.kind(), WorthQueryResourceStopKind::Unavailable);
        assert_eq!(
            fresh.resource.counts(),
            (0, 0, 0),
            "logical discharge never contacts the replacement provider"
        );
        let mut reacquired = resources.start::<Resource>(42).unwrap_or_else(|stop| {
            let (_, cause) = stop.into_parts();
            panic!("fresh declared Native receiver: {cause:?}")
        });
        assert_ne!(reacquired.obligation_identity(), Some(original));
        assert_eq!(fresh.resource.input(0), 42);
        resources.close(&mut reacquired).unwrap();
    }
    assert_eq!(
        fresh.resource.counts(),
        (1, 0, 1),
        "only explicitly reacquired work contacts the fresh provider"
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("cold logical discharge close: {cause:?}"))
        .reclaim();
}

#[path = "home_seed/handoff_publication_refusal.rs"]
mod handoff_publication_refusal;
