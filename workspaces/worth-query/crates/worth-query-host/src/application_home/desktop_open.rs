//! One-line Directory Home opening and clean retirement with explicit finite policy.
use std::fmt;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use worth_foundational::facade::{
    ApplicationBuildIdentity, DeterminismContract, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_declaration::facade::{
    application_program::{
        ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramOutputsShape,
        ApplicationProgramValidationDenial,
    },
    application_schema::{
        ApplicationSchema, ApplicationSchemaComposition, ApplicationSchemaDeclarationDenial,
    },
};
use worth_query_execution::facade::{
    application_contribution::{
        with_bootstrap_advancement, WorthQueryAdvancementDenial,
        WorthQueryApplicationContributionTuple, WorthQueryBootstrapAdvancementPhase,
    },
    application_installation::{
        WorthQueryApplicationLimits, WorthQueryApplicationProgramRoots,
        WorthQueryApplicationProgramRoster,
    },
    primary_graph::{
        WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
        WorthQueryRuntimeClock,
    },
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
use worth_runtime_world::facade::RuntimeWorldBudgetDenial;
use worth_store_application::{ApplicationHomeCheckpointLimits, ApplicationHomeCloseOutcome};

use super::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeCloseRefused,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeOpenRefused,
    WorthQueryApplicationHomeRequest, WorthQueryHomeClosePolicyRefused,
};

/// Explicit finite work ceiling used by default Host bootstrap, session, and
/// clean-close operations when the caller does not override the policy.
///
/// Reopening a Directory Home rebuilds its admission image from retained
/// history, and the first requests after a reopen may admit more of it. That
/// reconstructive work grows with the history the default limits retain
/// (about 5M units for a freshly seeded Home plus about 0.5M per retained
/// commit), so the ceiling must cover the full default retention and
/// checkpoint windows rather than one ordinary request. It stays finite.
pub const DEFAULT_HOST_EXECUTION_WORK_CEILING: u64 = 1 << 32;

/// Explicit bounded serial execution policy used by default Host DX helpers.
pub const DEFAULT_HOST_EXECUTION_POLICY: ExecutionRequestPolicy = ExecutionRequestPolicy::new(
    ExecutionPosture::Serial,
    DeterminismContract::CanonicalBitwise,
    ExecutionBudget::new(
        NonZeroUsize::MIN,
        None,
        Some(DEFAULT_HOST_EXECUTION_WORK_CEILING),
    ),
);

type InitialStateFn<'authoring, Schema> = Box<
    dyn FnOnce(
            &WorthQueryBootstrapAdvancementPhase<'_>,
            &mut WorthQueryPrimaryGraphBootstrap<Schema>,
            &WorthQueryInstalledApplicationSchema<Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
        + 'authoring,
>;

/// Configuration for [`WorthQueryApplicationHome::open_directory`] and
/// [`WorthQueryApplicationHome::open_memory`].
pub struct WorthQueryHomeConfig<'authoring, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    build: ApplicationBuildIdentity,
    configuration:
        <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    clock: WorthQueryAdmissionClock,
    execution_policy: ExecutionRequestPolicy,
    limits: Option<WorthQueryApplicationLimits>,
    checkpoint_limits: ApplicationHomeCheckpointLimits,
    roster: WorthQueryApplicationProgramRoster<'authoring, Schema>,
    authorization_time_source: Option<WorthQueryRuntimeClock>,
    capability_calendar: Option<
        std::sync::Arc<
            dyn worth_query_execution::facade::primary_graph::WorthQueryCapabilityCalendar,
        >,
    >,
    initial_state: InitialStateFn<'authoring, Schema>,
}

impl<'authoring, Schema, Program> WorthQueryHomeConfig<'authoring, Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    /// Starts a bounded serial desktop Home configuration with default
    /// contribution configuration, system clock, and finite budgets.
    pub fn desktop_default(build: ApplicationBuildIdentity) -> Self
    where
        <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration:
            Default,
    {
        Self::new(build, Default::default())
    }

    /// Starts a bounded serial Home configuration with an explicit contribution
    /// configuration tuple.
    pub fn new(
        build: ApplicationBuildIdentity,
        configuration: <Program::Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    ) -> Self {
        Self {
            build,
            configuration,
            clock: WorthQueryAdmissionClock::system(),
            execution_policy: DEFAULT_HOST_EXECUTION_POLICY,
            limits: None,
            checkpoint_limits: ApplicationHomeCheckpointLimits::default(),
            roster: WorthQueryApplicationProgramRoster::new(),
            authorization_time_source: None,
            capability_calendar: None,
            initial_state: Box::new(|_, _, _| Ok(())),
        }
    }

    /// Replaces the admission clock used by default bounded limits.
    pub fn with_clock(mut self, clock: WorthQueryAdmissionClock) -> Self {
        self.clock = clock;
        self
    }

    /// Replaces the bounded execution policy used during bootstrap open.
    pub const fn with_execution_policy(mut self, policy: ExecutionRequestPolicy) -> Self {
        self.execution_policy = policy;
        self
    }

    /// Replaces the default bounded serial application limits.
    pub fn with_limits(mut self, limits: WorthQueryApplicationLimits) -> Self {
        self.limits = Some(limits);
        self
    }

    /// Replaces the journal checkpoint thresholds.
    pub const fn with_checkpoint_limits(
        mut self,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
    ) -> Self {
        self.checkpoint_limits = checkpoint_limits;
        self
    }

    /// Supplies additional rostered programs supported alongside `Program`.
    pub fn with_roster(
        mut self,
        roster: WorthQueryApplicationProgramRoster<'authoring, Schema>,
    ) -> Self {
        self.roster = roster;
        self
    }

    /// Supplies the trusted clock used when admitting and checking capabilities.
    pub fn with_authorization_time_source(
        mut self,
        source: impl Into<WorthQueryRuntimeClock>,
    ) -> Self {
        self.authorization_time_source = Some(source.into());
        self
    }

    /// Supplies the calendar local-date grant validity is read on (for example
    /// `WorthQueryTimeZone::load_system()?`, or a fixed offset in tests). It
    /// joins the Home's trusted time basis: grants compare the local civil
    /// date of the one admission instant, never a caller-supplied date. A Home
    /// that installs a local-date grant without a calendar refuses to open.
    pub fn with_capability_calendar(
        mut self,
        calendar: impl worth_query_execution::facade::primary_graph::WorthQueryCapabilityCalendar,
    ) -> Self {
        self.capability_calendar = Some(std::sync::Arc::new(calendar));
        self
    }

    /// Supplies the one-time initial state seeder invoked only when creating a
    /// fresh Home (never replayed when reopening an existing Home).
    pub fn with_initial_state(
        mut self,
        initial_state: impl FnOnce(
                &WorthQueryBootstrapAdvancementPhase<'_>,
                &mut WorthQueryPrimaryGraphBootstrap<Schema>,
                &WorthQueryInstalledApplicationSchema<Schema>,
            ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
            + 'authoring,
    ) -> Self {
        self.initial_state = Box::new(initial_state);
        self
    }
}

/// Exact typed refusal from [`WorthQueryApplicationHome::open_directory`] or
/// [`WorthQueryApplicationHome::open_memory`], preserving any retained Home
/// opening custody intact.
#[must_use = "opening refusal may retain physical Home custody for recovery or close"]
pub enum WorthQueryDirectoryOpenDenial<Schema: ApplicationSchemaComposition, Program> {
    SchemaDeclaration(ApplicationSchemaDeclarationDenial),
    ProgramValidation(ApplicationProgramValidationDenial),
    Limits(RuntimeWorldBudgetDenial),
    Advancement(WorthQueryAdvancementDenial),
    Home(WorthQueryApplicationHomeOpenRefused<Schema, Program>),
}

impl<Schema: ApplicationSchemaComposition, Program> WorthQueryDirectoryOpenDenial<Schema, Program> {
    /// Borrows the underlying Home opening refusal when opening progressed past
    /// declaration, program validation, limits, and bootstrap admission.
    pub fn home_refusal(&self) -> Option<&WorthQueryApplicationHomeOpenRefused<Schema, Program>> {
        match self {
            Self::Home(refused) => Some(refused),
            _ => None,
        }
    }

    /// Unwraps the underlying Home opening refusal so callers can inspect
    /// recovery posture or close retained custody.
    pub fn into_home_refusal(
        self,
    ) -> Result<WorthQueryApplicationHomeOpenRefused<Schema, Program>, Self> {
        match self {
            Self::Home(refused) => Ok(refused),
            other => Err(other),
        }
    }
}

impl<Schema: ApplicationSchemaComposition, Program> fmt::Debug
    for WorthQueryDirectoryOpenDenial<Schema, Program>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaDeclaration(cause) => {
                f.debug_tuple("SchemaDeclaration").field(cause).finish()
            }
            Self::ProgramValidation(cause) => {
                f.debug_tuple("ProgramValidation").field(cause).finish()
            }
            Self::Limits(cause) => f.debug_tuple("Limits").field(cause).finish(),
            Self::Advancement(cause) => f.debug_tuple("Advancement").field(cause).finish(),
            Self::Home(cause) => f.debug_tuple("Home").field(cause).finish(),
        }
    }
}

impl<Schema: ApplicationSchemaComposition, Program> fmt::Display
    for WorthQueryDirectoryOpenDenial<Schema, Program>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaDeclaration(cause) => write!(f, "schema declaration failed: {cause:?}"),
            Self::ProgramValidation(cause) => write!(f, "program validation failed: {cause:?}"),
            Self::Limits(cause) => write!(f, "application limits refused: {cause:?}"),
            Self::Advancement(cause) => write!(f, "bootstrap advancement refused: {cause}"),
            Self::Home(cause) => write!(f, "home open refused: {cause}"),
        }
    }
}

impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    /// Opens or creates a persistent Directory Home in one call using explicit
    /// finite budgets. Inspect [`Self::open_disposition`] to distinguish fresh
    /// creation (`Created`) from reopening (`Opened`).
    pub fn open_directory<'authoring>(
        path: impl Into<PathBuf>,
        config: WorthQueryHomeConfig<'authoring, Schema, Program>,
    ) -> Result<Self, WorthQueryDirectoryOpenDenial<Schema, Program>> {
        Self::open_location(
            WorthQueryApplicationHomeLocation::Directory(path.into()),
            config,
        )
    }

    /// Opens an in-memory Home in one call using the same configuration builder.
    pub fn open_memory<'authoring>(
        config: WorthQueryHomeConfig<'authoring, Schema, Program>,
    ) -> Result<Self, WorthQueryDirectoryOpenDenial<Schema, Program>> {
        Self::open_location(WorthQueryApplicationHomeLocation::MemoryOnly, config)
    }

    fn open_location<'authoring>(
        location: WorthQueryApplicationHomeLocation,
        config: WorthQueryHomeConfig<'authoring, Schema, Program>,
    ) -> Result<Self, WorthQueryDirectoryOpenDenial<Schema, Program>> {
        use WorthQueryDirectoryOpenDenial as Denial;
        let declaration = Schema::declaration().map_err(Denial::SchemaDeclaration)?;
        let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .map_err(Denial::ProgramValidation)?;
        let admission_clock = config.clock.clone();
        let limits = match config.limits {
            Some(limits) => limits,
            None => WorthQueryApplicationLimits::bounded_serial(config.clock)
                .with_execution_policy(config.execution_policy)
                .install()
                .map_err(Denial::Limits)?,
        };
        let mut request = WorthQueryApplicationHomeRequest::new(
            location,
            program,
            config.roster,
            declaration,
            config.configuration,
            config.build,
            limits,
            config.initial_state,
        )
        .with_checkpoint_limits(config.checkpoint_limits);
        let source = config.authorization_time_source.unwrap_or_else(|| {
            WorthQueryRuntimeClock::system().with_admission_clock(admission_clock)
        });
        request.authorization_time_source = Some(match config.capability_calendar {
            Some(calendar) => super::capability_calendar::with_calendar(Some(source), calendar),
            None => source,
        });
        let home = with_bootstrap_advancement(config.execution_policy, |phase| {
            request.open_in_advancement(&phase)
        })
        .map_err(Denial::Advancement)?
        .map_err(Denial::Home)?;
        Ok(home)
    }
}

impl<Schema: ApplicationSchema, Program: 'static> WorthQueryApplicationHome<Schema, Program> {
    /// Cleanly retires this Directory Home under [`DEFAULT_HOST_EXECUTION_POLICY`].
    pub fn close_clean(
        self,
    ) -> Result<
        ApplicationHomeCloseOutcome,
        WorthQueryHomeClosePolicyRefused<
            Self,
            WorthQueryApplicationHomeCloseRefused<Schema, Program>,
        >,
    > {
        self.close_clean_with_policy(DEFAULT_HOST_EXECUTION_POLICY)
    }

    /// Cleanly retires this Directory Home under an explicit execution policy.
    pub fn close_clean_with_policy(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<
        ApplicationHomeCloseOutcome,
        WorthQueryHomeClosePolicyRefused<
            Self,
            WorthQueryApplicationHomeCloseRefused<Schema, Program>,
        >,
    > {
        let (outcome, retired_query) = self.close_with_policy(policy)?.into_parts();
        if let Some(retired) = retired_query {
            retired.reclaim();
        }
        Ok(outcome)
    }
}
