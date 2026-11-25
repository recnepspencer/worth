//! Single-call Directory Home open-with-transition entry points.

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

use crate::application_home::WorthQueryApplicationHomeRequest;
use worth_execution::ExecutionMemoryReservation;
use worth_query_declaration::facade::{
    application_program::{ApplicationProgramDefinition, ApplicationProgramOutputsShape},
    program_transition::{
        WorthQueryProgramTransitionIndexRebuild, WorthQueryProgramTransitionSpec,
    },
};
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryApplicationContributionTuple, WorthQueryBootstrapAdvancementPhase,
    },
    application_home_integration::{
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
    application_installation::WorthQueryApplicationProgramRoots,
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

use super::{
    config::WorthQueryProgramTransitionOpenConfig,
    outcome::{
        WorthQueryProgramTransitionOpenDenial, WorthQueryTransitionOpenOutcome,
        WorthQueryTransitionOpenedHome,
    },
    planner::{compile_declarative_plan, PlanInput},
};

/// Open a Directory Home on `T::ToProgram`, upgrading atomically in-place from
/// `T::FromProgram` when the on-disk Home is on `T::FromProgram`.
///
/// - Already on `T::ToProgram` -> returns [`WorthQueryTransitionOpenOutcome::OpenedCurrent`]
///   without running any transition or fresh initializer.
/// - Fresh directory -> returns [`WorthQueryTransitionOpenOutcome::CreatedFresh`]
///   without running any predecessor or transition.
/// - Existing `T::FromProgram` -> executes `T` atomically in-place and returns
///   [`WorthQueryTransitionOpenOutcome::UpgradedInPlace`].
/// - Downgrade, non-advancing version, wrong predecessor, invalid config, or
///   budget/cancellation stop -> fails closed with a typed
///   [`WorthQueryProgramTransitionOpenDenial`] leaving disk uncorrupted.
///
/// ```ignore
/// let opened = open_directory_with_transition::<DaymarkV2ToV3>(
///     &phase,
///     v2_request(path.clone(), build),
///     v3_request(path, build),
///     WorthQueryProgramTransitionOpenConfig::default_bounded(),
/// )?;
/// let (home, outcome) = opened.into_parts();
/// ```
pub fn open_directory_with_transition<'a, T>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    predecessor: WorthQueryApplicationHomeRequest<'a, T::FromSchema, T::FromProgram>,
    successor: WorthQueryApplicationHomeRequest<'a, T::ToSchema, T::ToProgram>,
    config: WorthQueryProgramTransitionOpenConfig,
) -> Result<
    WorthQueryTransitionOpenedHome<T::ToSchema, T::ToProgram>,
    WorthQueryProgramTransitionOpenDenial<T::FromSchema, T::FromProgram, T::ToSchema, T::ToProgram>,
>
where
    T: WorthQueryProgramTransitionSpec,
    <T::FromProgram as ApplicationProgramDefinition<T::FromSchema>>::Outputs:
        ApplicationProgramOutputsShape<T::FromSchema>
            + WorthQueryApplicationProgramRoots<T::FromSchema>,
    <T::FromProgram as ApplicationProgramDefinition<T::FromSchema>>::Contributions:
        WorthQueryApplicationContributionTuple<T::FromSchema>,
    <T::ToProgram as ApplicationProgramDefinition<T::ToSchema>>::Outputs:
        ApplicationProgramOutputsShape<T::ToSchema>
            + WorthQueryApplicationProgramRoots<T::ToSchema>,
    <T::ToProgram as ApplicationProgramDefinition<T::ToSchema>>::Contributions:
        WorthQueryApplicationContributionTuple<T::ToSchema>,
{
    open_directory_with_transition_and_step::<T, _>(
        phase,
        (predecessor, successor),
        config,
        |_, _| Ok(Vec::new()),
    )
}

/// Same as [`open_directory_with_transition`], plus an optional typed custom
/// migration step appended to the declarative target step.
pub fn open_directory_with_transition_and_step<'a, T, CustomStep>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    requests: (
        WorthQueryApplicationHomeRequest<'a, T::FromSchema, T::FromProgram>,
        WorthQueryApplicationHomeRequest<'a, T::ToSchema, T::ToProgram>,
    ),
    config: WorthQueryProgramTransitionOpenConfig,
    custom_step: CustomStep,
) -> Result<
    WorthQueryTransitionOpenedHome<T::ToSchema, T::ToProgram>,
    WorthQueryProgramTransitionOpenDenial<T::FromSchema, T::FromProgram, T::ToSchema, T::ToProgram>,
>
where
    T: WorthQueryProgramTransitionSpec,
    <T::FromProgram as ApplicationProgramDefinition<T::FromSchema>>::Outputs:
        ApplicationProgramOutputsShape<T::FromSchema>
            + WorthQueryApplicationProgramRoots<T::FromSchema>,
    <T::FromProgram as ApplicationProgramDefinition<T::FromSchema>>::Contributions:
        WorthQueryApplicationContributionTuple<T::FromSchema>,
    <T::ToProgram as ApplicationProgramDefinition<T::ToSchema>>::Outputs:
        ApplicationProgramOutputsShape<T::ToSchema>
            + WorthQueryApplicationProgramRoots<T::ToSchema>,
    <T::ToProgram as ApplicationProgramDefinition<T::ToSchema>>::Contributions:
        WorthQueryApplicationContributionTuple<T::ToSchema>,
    CustomStep: FnOnce(
        &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
        &WorthQueryHomeProgramTransitionTarget<'_>,
    )
        -> Result<Vec<WorkerIntentBatch>, WorthQueryPrimaryGraphInstallationDenial>,
{
    let descriptor = T::descriptor().map_err(WorthQueryProgramTransitionOpenDenial::Declaration)?;
    descriptor
        .validate_for_spec::<T>()
        .map_err(WorthQueryProgramTransitionOpenDenial::Declaration)?;
    config
        .validate()
        .map_err(WorthQueryProgramTransitionOpenDenial::InvalidConfiguration)?;

    let (predecessor, successor) = requests;
    let (resources, max_entities_per_kind, dispositions) = config.into_parts();
    let observers = OpenObservers::new();
    let record_handle = observers.recorder();
    // Retain backfill memory reservations until `open_in_advancement` finishes
    // consuming and committing the planned `WorkerIntentBatch`es.
    let retained_memory: Arc<Mutex<Vec<ExecutionMemoryReservation>>> =
        Arc::new(Mutex::new(Vec::new()));
    let retained_for_plan = Arc::clone(&retained_memory);

    let mut request = successor
        .observe_initial_state(observers.created_fresh.clone())
        .with_program_transition(predecessor, resources, move |source, target| {
            let input = PlanInput {
                descriptor: &descriptor,
                max_entities_per_kind,
                retained_memory: &retained_for_plan,
            };
            let compiled = compile_declarative_plan(input, source, target, custom_step)?;
            record_handle.record(
                compiled.backfilled_records,
                compiled.verified_index_rebuilds,
            );
            Ok(compiled.plan)
        });
    if let Some(dispositions) = dispositions {
        request = request.with_workflow_dispositions(dispositions);
    }
    let opened = request
        .open_in_advancement(phase)
        .map_err(WorthQueryProgramTransitionOpenDenial::classify_host);
    drop(retained_memory);
    let home = opened?;
    Ok(WorthQueryTransitionOpenedHome::new(
        home,
        observers.into_outcome::<T>(),
    ))
}

struct UpgradeRecorder {
    upgraded: Arc<AtomicBool>,
    backfilled: Arc<AtomicUsize>,
    verified_indexes: Arc<Mutex<Vec<WorthQueryProgramTransitionIndexRebuild>>>,
}

impl UpgradeRecorder {
    fn record(
        &self,
        backfilled_records: usize,
        indexes: Vec<WorthQueryProgramTransitionIndexRebuild>,
    ) {
        self.upgraded.store(true, Ordering::SeqCst);
        self.backfilled.store(backfilled_records, Ordering::SeqCst);
        if let Ok(mut guard) = self.verified_indexes.lock() {
            *guard = indexes;
        }
    }
}

struct OpenObservers {
    created_fresh: Arc<AtomicBool>,
    upgraded: Arc<AtomicBool>,
    backfilled: Arc<AtomicUsize>,
    verified_indexes: Arc<Mutex<Vec<WorthQueryProgramTransitionIndexRebuild>>>,
}

impl OpenObservers {
    fn new() -> Self {
        Self {
            created_fresh: Arc::new(AtomicBool::new(false)),
            upgraded: Arc::new(AtomicBool::new(false)),
            backfilled: Arc::new(AtomicUsize::new(0)),
            verified_indexes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn recorder(&self) -> UpgradeRecorder {
        UpgradeRecorder {
            upgraded: self.upgraded.clone(),
            backfilled: self.backfilled.clone(),
            verified_indexes: self.verified_indexes.clone(),
        }
    }

    fn into_outcome<T: WorthQueryProgramTransitionSpec>(self) -> WorthQueryTransitionOpenOutcome {
        if self.upgraded.load(Ordering::SeqCst) {
            let indexes_marked_for_rebuild = self
                .verified_indexes
                .lock()
                .map(|guard| guard.clone())
                .unwrap_or_default();
            WorthQueryTransitionOpenOutcome::UpgradedInPlace {
                from_version: T::FROM_VERSION,
                to_version: T::TO_VERSION,
                backfilled_records: self.backfilled.load(Ordering::SeqCst),
                indexes_marked_for_rebuild,
            }
        } else if self.created_fresh.load(Ordering::SeqCst) {
            WorthQueryTransitionOpenOutcome::CreatedFresh {
                program_version: T::TO_VERSION,
            }
        } else {
            WorthQueryTransitionOpenOutcome::OpenedCurrent {
                program_version: T::TO_VERSION,
            }
        }
    }
}
