//! Application program, desktop temporal contribution installation, and Home helpers.

mod runtime_ops;
use runtime_ops::seed_reminder_graph;
pub use runtime_ops::{mutate_reminder, read_reminders, unique_temp_dir};

use std::{marker::PhantomData, path::PathBuf, sync::Arc};
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
};
use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryApplicationConditionalBinding,
        WorthQueryApplicationConditionalPackageContract,
        WorthQueryApplicationConditionalProducerAccess, WorthQueryApplicationContribution,
        WorthQueryApplicationContributionContracts, WorthQueryApplicationContributionSetup,
        WorthQueryOutputReadinessContractDenial,
    },
    application_installation::{
        ApplicationBuildIdentity, WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
    },
    domain, primary_graph,
    temporal_contribution::{
        install_desktop_temporal_clock_operation, WorthQueryAlwaysReadyTemporalPredicate,
        WorthQueryDesktopClockSource, WorthQueryDesktopTemporalBindingSpec,
        WorthQueryInstalledDesktopTemporalClock,
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
    pub fire_log: ReminderFireLog,
    pub auth: AuthControl,
    pub controls: InvokerControls,
    pub use_mismatched_due_field: bool,
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
        let temporal = if config.use_mismatched_due_field {
            install_desktop_temporal_clock_operation(
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
                        controls: config.controls.clone(),
                    },
                    identity_field: identity_field(),
                    revision_field: revision_field(),
                    lifecycle_field: lifecycle_field(),
                    active_lifecycle: "active".to_owned(),
                    completed_lifecycle: "completed".to_owned(),
                    due_field: primary_graph::WorthQueryTemporalDueField::new(interval_field()),
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
            .expect("mismatched due field desktop temporal installs")
        } else {
            install_desktop_temporal_clock_operation(
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
                        controls: config.controls.clone(),
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
            .expect("desktop temporal operation installs cleanly")
        };
        Ok(temporal)
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
        setup.handler::<MutateReminderBinding, _>(MutateReminderHandler {
            controls: configuration.controls.clone(),
        })?;
        setup.conditional::<DesktopReminderConditional>(configuration)
    }
}

pub fn open_memory_home_with_mismatched_due_field(
    fire_log: ReminderFireLog,
    auth: AuthControl,
    execution: &ExecutionRequest<'_, '_>,
) -> ReminderHome {
    open_home_at(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        fire_log,
        auth,
        InvokerControls::default(),
        true,
        None,
        execution,
    )
}

pub fn open_memory_home_with_controls(
    fire_log: ReminderFireLog,
    auth: AuthControl,
    controls: InvokerControls,
    limits: Option<WorthQueryApplicationLimits>,
    execution: &ExecutionRequest<'_, '_>,
) -> ReminderHome {
    open_home_at(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        fire_log,
        auth,
        controls,
        false,
        limits,
        execution,
    )
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
        InvokerControls::default(),
        false,
        None,
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
        InvokerControls::default(),
        false,
        None,
        execution,
    )
}

fn open_home_at(
    location: WorthQueryApplicationHomeLocation,
    fire_log: ReminderFireLog,
    auth: AuthControl,
    controls: InvokerControls,
    use_mismatched_due_field: bool,
    custom_limits: Option<WorthQueryApplicationLimits>,
    execution: &ExecutionRequest<'_, '_>,
) -> ReminderHome {
    let (clock_source, _clock_handle) = WorthQueryDesktopClockSource::new(
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
        fire_log,
        auth,
        controls,
        use_mismatched_due_field,
    };
    let limits = custom_limits.unwrap_or_else(|| {
        WorthQueryApplicationLimits::ordinary_serial(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        )
        .with_signal_budget(primary_graph::SignalConditionalEvaluationBudget::development())
    });
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

pub fn installed(home: &ReminderHome) -> Arc<InstalledDesktopReminder> {
    home.runtime()
        .conditional::<DesktopReminderConditional>()
        .expect("desktop reminder conditional installed")
}
