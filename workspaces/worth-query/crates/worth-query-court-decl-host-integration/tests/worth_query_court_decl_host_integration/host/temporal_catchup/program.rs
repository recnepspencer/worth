//! Application program, desktop temporal contribution installation, and Home helpers.

use std::{
    collections::BTreeMap,
    marker::PhantomData,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use worth_execution::ExecutionRequest;
use worth_query_decl::facade::{
    application_program::{
        ApplicationActionDelivery, ApplicationFeature, ApplicationFeatureInputLeaf,
        ApplicationFeatureSpec, ApplicationNoOutputGraph, ApplicationProgramAuthoring,
        ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
        ApplicationRuleLeaf, ApplicationSourceCompletion, ValidatedApplicationProgram,
    },
    application_query::ApplicationQueryParameterSet,
    application_schema::{
        ApplicationSchema, ApplicationSchemaComposition, ErasedApplicationSchemaDeclaration,
    },
    authentication::{WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus},
};
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryApplicationConditionalBinding,
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryApplicationConditionalProducerAccess, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
        WorthQueryOutputReadinessContractDenial,
    },
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    domain, primary_graph, runtime,
    temporal_contribution::{
        install_desktop_temporal_operation, WorthQueryAlwaysReadyTemporalPredicate,
        WorthQueryDesktopClockHandle, WorthQueryDesktopClockSource,
        WorthQueryDesktopTemporalBindingSpec, WorthQueryInstalledDesktopTemporalClock,
    },
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

use super::{adapters::*, schema::*};

pub type ReminderHome = WorthQueryApplicationHome<DesktopReminderSchema, DesktopReminderProgram>;

pub struct DesktopReminderProgram;
pub struct DesktopReminderFeature;
pub struct MutateReminderCompletion;
impl ApplicationActionDelivery<DesktopReminderSchema, MutateReminderBinding>
    for MutateReminderCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "worth.query.court.desktop-reminder.mutate-completion.v1";
}
impl ApplicationFeature<DesktopReminderSchema> for DesktopReminderFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.court.desktop-reminder.feature.v1";
}
impl ApplicationProgramDefinition<DesktopReminderSchema> for DesktopReminderProgram {
    type Contributions = <DesktopReminderSchema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.court.desktop-reminder.program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DesktopReminderSchema, DesktopReminderFeature>()
                .conditional_operation::<FireReminder>()
                .action::<MutateReminderBinding, MutateReminderCompletion>()
                .finish_action()
                .finish(),
        ]
    }
}

pub fn validated_program(
) -> ValidatedApplicationProgram<DesktopReminderSchema, DesktopReminderProgram> {
    ApplicationProgramAuthoring::<DesktopReminderSchema, DesktopReminderProgram>::begin()
        .validated_program()
        .expect("desktop reminder program validates")
}

pub struct DesktopReminderConfig {
    pub clock_source: WorthQueryDesktopClockSource<DesktopReminderClock>,
    pub clock_handle: WorthQueryDesktopClockHandle<DesktopReminderClock>,
    pub fire_log: ReminderFireLog,
    pub auth: AuthControl,
}

pub type InstalledDesktopReminder = WorthQueryInstalledDesktopTemporalClock<
    DesktopReminderSchema,
    DesktopReminderNode,
    DesktopReminderClock,
>;

pub struct DesktopReminderConditional;
impl WorthQueryApplicationConditionalBinding<DesktopReminderSchema> for DesktopReminderConditional {
    type Configuration = DesktopReminderConfig;
    type Installed = InstalledDesktopReminder;
    type Operation = FireReminder;
    const IDENTITY: &'static str = "worth.query.court.desktop-reminder.conditional.v1";
    const REQUIRED_PRODUCERS: &'static [&'static str] = &[];

    fn package_contract(
        declaration: &ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryOutputReadinessContractDenial,
    > {
        desktop_node_spec().package_contract(
            declaration,
            FireReminder::reference(),
            DesktopReminderNode::reference(),
        )
    }

    fn install(
        config: Self::Configuration,
        _: &WorthQueryApplicationConditionalProducerAccess<'_, DesktopReminderSchema>,
        installation: &mut primary_graph::WorthQueryConditionalApplicationRuntimeInstallation<
            DesktopReminderSchema,
        >,
    ) -> Result<Self::Installed, primary_graph::WorthQueryConditionalRuntimeInstallationDenial>
    {
        let declaration = DesktopReminderSchema::declaration().unwrap();
        let conditional_binding = desktop_node_spec()
            .conditional_binding(declaration.erased(), FireReminder::reference())
            .expect("desktop temporal node spec binds");
        let adapter = Arc::new(admit_local_adapter(installation.installed_schema()));
        let temporal = install_desktop_temporal_operation(
            installation,
            WorthQueryDesktopTemporalBindingSpec {
                operation: FireReminder::reference(),
                conditional_binding,
                node: DesktopReminderNode::reference(),
                predicate: WorthQueryAlwaysReadyTemporalPredicate,
                clock_source: config.clock_source,
                parameters: ApplicationQueryParameterSet::new(),
                projector: ReminderProjector,
                bounds: domain::WorthQueryTemporalIntentBounds::new(8, 16, 8).unwrap(),
                principal_binding: ReminderPrincipalBinding::reference(),
                principal_source: DesktopPrincipalSource {
                    adapter,
                    auth: config.auth.clone(),
                },
                scope_field: identity_field(),
                scope_value: "hub".to_owned(),
                invoker: ReminderInvoker {
                    fire_log: config.fire_log.clone(),
                },
                identity_field: identity_field(),
                revision_field: revision_field(),
                lifecycle_field: lifecycle_field(),
                active_lifecycle: "active".to_owned(),
                completed_lifecycle: "completed".to_owned(),
                due_field: primary_graph::WorthQueryTemporalDueField::new(due_field()),
                authorization: primary_graph::WorthQueryPublicTemporalOperationAuthorization,
                query_authorization: primary_graph::WorthQueryPublicTemporalQueryAuthorization,
                _marker: PhantomData::<
                    fn() -> (
                        DesktopReminderClock,
                        ReminderTargetsReadBinding,
                        ReminderIntent,
                        ReminderIntent,
                    ),
                >,
            },
        )
        .expect("desktop temporal operation installs cleanly");
        Ok(temporal.into_home_binding(config.clock_handle))
    }
}

impl WorthQueryApplicationContribution<DesktopReminderSchema> for DesktopReminderContribution {
    type Configuration = DesktopReminderConfig;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<DesktopReminderSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.conditional::<DesktopReminderConditional>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DesktopReminderSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<MutateReminderBinding, _>(MutateReminderHandler)?;
        setup.conditional::<DesktopReminderConditional>(configuration)
    }
}

pub fn open_memory_home(
    fire_log: ReminderFireLog,
    auth: AuthControl,
    execution: &ExecutionRequest<'_, '_>,
) -> ReminderHome {
    open_home_at(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        fire_log,
        auth,
        execution,
    )
}

pub fn open_directory_home(
    location: PathBuf,
    fire_log: ReminderFireLog,
    auth: AuthControl,
    execution: &ExecutionRequest<'_, '_>,
) -> ReminderHome {
    open_home_at(
        WorthQueryApplicationHomeLocation::Directory(location),
        fire_log,
        auth,
        execution,
    )
}

fn open_home_at(
    location: WorthQueryApplicationHomeLocation,
    fire_log: ReminderFireLog,
    auth: AuthControl,
    execution: &ExecutionRequest<'_, '_>,
) -> ReminderHome {
    let (clock_source, clock_handle) = WorthQueryDesktopClockSource::new(
        domain::WorthQueryClockSourceIdentity::declare(
            "worth.query.court.desktop-reminder.clock-source",
        )
        .expect("clock source identity valid"),
        domain::WorthQueryClockTimelineIdentity::declare(
            "worth.query.court.desktop-reminder.timeline",
        )
        .expect("clock timeline identity valid"),
        domain::WorthQueryClockCoordinate::from_nanoseconds(0),
    );
    let config = DesktopReminderConfig {
        clock_source,
        clock_handle,
        fire_log,
        auth,
    };
    let limits = WorthQueryApplicationLimits::ordinary_serial(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
    .with_signal_budget(primary_graph::SignalConditionalEvaluationBudget::development());
    let request = WorthQueryApplicationHomeRequest::new(
        location,
        validated_program(),
        WorthQueryApplicationProgramRoster::new(),
        DesktopReminderSchema::declaration().unwrap(),
        (config,),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        limits,
        |_, graph, installed| {
            let principal = installed
                .principal_binding(ReminderPrincipalBinding::reference())
                .unwrap();
            seed_reminder_graph(graph, &principal);
            Ok(())
        },
    );
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .expect("bootstrap advancement opens")
        .unwrap_or_else(|denial| panic!("desktop reminder Home open failed: {denial:?}"))
}

fn seed_reminder_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<DesktopReminderSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        DesktopReminderSchema,
        ReminderPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        worth_query_decl::facade::application_schema::U64ApplicationValueBinding,
    >,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("daymark-user").unwrap(),
            1_u64,
            WorthQueryExternalPrincipalIdentity::new("https://local.desktop", "daymark-user")
                .unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    for (row_key, id) in [("row-hub", "hub"), ("row-i1", "i1"), ("row-i2", "i2")] {
        graph
            .bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    ReminderIntent::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(row_key).unwrap(),
                )
                .field(identity_field(), id.to_owned())
                .field(revision_field(), 0_u64)
                .field(due_field(), 0_u64)
                .field(interval_field(), 0_u64)
                .field(max_fires_field(), 0_u64)
                .field(fired_count_field(), 0_u64)
                .field(lifecycle_field(), "absent".to_owned())
                .field(gate_field(), "ready".to_owned()),
            )
            .unwrap();
    }
    for (rel_id, target_key) in [("hub-to-i1", "row-i1"), ("hub-to-i2", "row-i2")] {
        graph
            .bind_relation(primary_graph::WorthQueryApplicationRelationSeed::new(
                ReminderTarget::reference(),
                rel_id,
                primary_graph::WorthQueryApplicationEntityKey::new("row-hub").unwrap(),
                primary_graph::WorthQueryApplicationEntityKey::new(target_key).unwrap(),
            ))
            .unwrap();
    }
}

pub fn installed(home: &ReminderHome) -> Arc<InstalledDesktopReminder> {
    home.runtime()
        .conditional::<DesktopReminderConditional>()
        .expect("desktop reminder conditional installed")
}

pub fn mutate_reminder(home: &ReminderHome, input: MutateReminderInput, command_key: u64) {
    let runtime = home.runtime();
    let scope = fresh_request_scope();
    let adapter = admit_local_adapter(runtime.installed_schema());
    let principal = adapter.authenticate_local(&scope).unwrap();
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(MutateReminderIntent(input))
        .without_source()
        .idempotency(&command_key)
        .execute_in_program(
            runtime,
            runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("reminder mutation executes");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "expected committed reminder mutation, got {outcome:?}"
    );
}

pub fn read_reminders(home: &ReminderHome) -> BTreeMap<String, ReminderRow> {
    let runtime = home.runtime();
    let scope = fresh_request_scope();
    let adapter = admit_local_adapter(runtime.installed_schema());
    let principal = adapter.authenticate_local(&scope).unwrap();
    let result = runtime
        .request(&principal, &scope)
        .query(ReminderTargetsReadRequest {
            hub: "hub".to_owned(),
        })
        .execute()
        .expect("reminder targets query executes");
    result
        .rows()
        .iter()
        .cloned()
        .map(|row| (row.identity.clone(), row))
        .collect()
}

pub fn unique_temp_dir(tag: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "worth-temporal-catchup-{tag}-{}-{seq}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}
