pub use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::UNIX_EPOCH;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

fn unique_dir(label: &str) -> std::path::PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "worth-host-{label}-{}-{}-{}",
        std::process::id(),
        worth_execution::time::system_time_sampler()
            .system_time()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    ))
}

pub fn application_checkpoint_restores_fresh_editable_authority(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let dir = unique_dir("temporal-restore");
    let lease = crate::facade::host::checkpoint_capture::lease(128 * 1024 * 1024);
    let open_execution = worth_execution::ExecutionRequest::leased(&lease);

    let contacts = ContactCounters::default();
    let (installation_predicate, _) = Predicate::controlled(contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let home = worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request(
        open_execution,
        |phase| {
            WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(dir.clone()),
                validated_program(),
                application_installation::WorthQueryApplicationProgramRoster::new(),
                TemporalHostSchema::declaration().unwrap(),
                (TemporalContributionConfiguration {
                    installation_predicate,
                    definition_predicate: Arc::new(definition_predicate),
                    clock_source,
                    clock_control,
                    contacts,
                    install_route: true,
                    definition_only: false,
                    multi_intent_targets: false,
                },),
                ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
                checkpoint_limits(),
                |_phase, graph, installed| {
                    let principal = installed
                        .principal_binding(TemporalPrincipalBinding::reference())
                        .unwrap();
                    seed_graph(graph, &principal, "checkpointed", 0, 1, true);
                    Ok(())
                },
            )
            .open_in_advancement(&phase)
        },
    )
    .expect("bootstrap opens")
    .expect("the checkpoint source Directory Home opens");
    assert_eq!(home.runtime().checkpoint_restore_work(), None);
    let original_world = home.runtime().current_world();
    assert_live_relation_is_readable(home.runtime(), "source");
    let _ = home
        .checkpoint(&open_execution)
        .expect("explicit Directory Home checkpoint publishes");
    home.close(&open_execution)
        .expect("Directory Home closes cleanly")
        .reclaim();

    let restored_contacts = ContactCounters::default();
    let restored_execution_probe = restored_contacts.clone();
    let (installation_predicate, _) = Predicate::controlled(restored_contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(restored_contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let restored_home = worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request(
        open_execution,
        |phase| {
            WorthQueryApplicationHomeRequest::new(
                WorthQueryApplicationHomeLocation::Directory(dir.clone()),
                validated_program(),
                application_installation::WorthQueryApplicationProgramRoster::new(),
                TemporalHostSchema::declaration().unwrap(),
                (TemporalContributionConfiguration {
                    installation_predicate,
                    definition_predicate: Arc::new(definition_predicate),
                    clock_source,
                    clock_control,
                    contacts: restored_contacts,
                    install_route: true,
                    definition_only: false,
                    multi_intent_targets: false,
                },),
                ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
                checkpoint_limits(),
                |_, _, _| panic!("reopen cannot run initial_state"),
            )
            .open_in_advancement(&phase)
        },
    )
    .expect("bootstrap opens")
    .expect("the checkpointed Directory Home reopens");
    let restored = restored_home.runtime();
    let restore_work = restored
        .checkpoint_restore_work()
        .expect("native readmission reports its actual restore work");
    assert!(restore_work.root_images_verified > 0);
    assert!(restore_work.branch_cells_readmitted > 0);
    assert_eq!(
        restored_execution_probe.snapshot(),
        (0, 0, 0, 0),
        "restore must not execute conditional application work",
    );
    assert_ne!(restored.current_world(), original_world);
    assert_live_relation_is_readable(restored, "restored");
    {
        let installed = restored
            .conditional::<TemporalConditional>()
            .expect("the restored contribution remains installed");
        let installed = installed
            .as_ref()
            .as_ref()
            .expect("the restored conditional route is live");
        change_input(
            request_execution,
            restored,
            &installed.invariant,
            restored.current_world(),
            "post-restore-edit",
        )
        .require_committed()
        .expect("fresh restored authority accepts a subsequent edit");
    }
    restored_home
        .close(&open_execution)
        .expect("reopened Directory Home closes cleanly")
        .reclaim();
    let _ = std::fs::remove_dir_all(&dir);
}

pub fn assert_live_relation_is_readable(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        TemporalInstallationProgram,
    >,
    phase: &str,
) {
    let request = request_scope();
    let schema = application.installed_schema();
    let principal_binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = admit_identity_adapter(schema);
    let external = block_on(authentication.authenticate((), &request)).unwrap();
    let selected = application
        .with_host_advancement(|host_phase| {
            application
                .on_branch(application.current_world())
                .select(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let principal = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_authenticated_principal(
                    &host_phase,
                    &principal_binding,
                    &external,
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let scope = application
        .with_host_advancement(|host_phase| {
            selected
                .resolve_entity(
                    &host_phase,
                    IntentIdentityField::at("IntentIdentityField"),
                    "intent-1".to_owned(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let query = schema
        .certification_query(TemporalIntentLiveQuery::reference())
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = application.with_host_advancement(|host_phase| selected
        .admit_application_query(
            &host_phase, &query,
            &access,
            worth_query_host::facade::declaration::application_query::ApplicationQueryParameterSet::new(),
            primary_graph::WorthQueryProductQueryControls::new(std::num::NonZeroUsize::new(8).unwrap(), &request),
        )
        .unwrap()).expect("installed host call opens");
    let result = application
        .with_host_advancement(|phase| {
            application
                .execute_application_query_one_shot(&phase, plan)
                .expect("checkpointed relation remains readable")
        })
        .expect("installed host call opens");
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].targets, vec!["intent-1"], "{phase}");
}

pub fn checkpoint_limits() -> WorthQueryApplicationLimits {
    WorthQueryApplicationLimits::new(
        product_world_resources(1_024),
        runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
        ),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
