//! Original Query custody is admitted before World seal and retained afterward.
use super::runtime::*;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_installation::{
        WorthQueryPreparedHomeQuery, WorthQueryProgramApplicationRuntime,
        WorthQueryRetiredHomeQuery, WorthQueryRetiringHomeQuery,
    },
    primary_graph::WorthQueryHomeRetirementDenial,
};
use worth_store_application::*;
mod maintenance;
mod retirement;
mod retry;

pub struct WorthQueryApplicationHomeClosed {
    outcome: ApplicationHomeCloseOutcome,
    retired_query: Option<WorthQueryRetiredHomeQuery>,
}
impl WorthQueryApplicationHomeClosed {
    pub fn is_closed(&self) -> bool {
        matches!(self.outcome, ApplicationHomeCloseOutcome::Closed { .. })
    }
    pub(super) fn into_parts(
        self,
    ) -> (
        ApplicationHomeCloseOutcome,
        Option<WorthQueryRetiredHomeQuery>,
    ) {
        (self.outcome, self.retired_query)
    }
    pub fn outcome(&self) -> &ApplicationHomeCloseOutcome {
        &self.outcome
    }
    /// Arbitrary inert supplier destruction runs only outside budgeted close.
    pub fn reclaim(self) {
        let (outcome, retired_query) = self.into_parts();
        drop(outcome);
        if let Some(retired_query) = retired_query {
            retired_query.reclaim();
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationHomeClosePhase {
    MemoryLifecycle,
    WorldAdmission,
    QueryAdmission,
    WorldClose,
    QueryRetirement,
    MaintenanceRetirement,
    PhysicalClose,
}

impl WorthQueryApplicationHomeClosePhase {
    const fn label(self) -> &'static str {
        match self {
            Self::MemoryLifecycle => "memory-lifecycle",
            Self::WorldAdmission => "world-admission",
            Self::QueryAdmission => "query-admission",
            Self::WorldClose => "world-close",
            Self::QueryRetirement => "query-retirement",
            Self::MaintenanceRetirement => "maintenance-retirement",
            Self::PhysicalClose => "physical-close",
        }
    }
}

pub struct WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    custody: Box<CloseCustody<Schema, Program>>,
}
impl<Schema, Program> WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    #[inline(never)]
    fn from_custody(custody: CloseCustody<Schema, Program>) -> Self {
        Self {
            custody: Box::new(custody),
        }
    }

    pub fn phase(&self) -> WorthQueryApplicationHomeClosePhase {
        match self.custody.as_ref() {
            CloseCustody::MemoryLifecycleRequired(_) => {
                WorthQueryApplicationHomeClosePhase::MemoryLifecycle
            }
            CloseCustody::WorldAdmission { .. } => {
                WorthQueryApplicationHomeClosePhase::WorldAdmission
            }
            CloseCustody::QueryAdmission { .. } => {
                WorthQueryApplicationHomeClosePhase::QueryAdmission
            }
            CloseCustody::World { .. } => WorthQueryApplicationHomeClosePhase::WorldClose,
            CloseCustody::QueryRetirement { .. } => {
                WorthQueryApplicationHomeClosePhase::QueryRetirement
            }
            CloseCustody::MaintenanceRetirement { .. } => {
                WorthQueryApplicationHomeClosePhase::MaintenanceRetirement
            }
            CloseCustody::QueryReleased { .. } => {
                WorthQueryApplicationHomeClosePhase::PhysicalClose
            }
        }
    }
}
enum CloseCustody<Schema, Program> {
    MemoryLifecycleRequired(WorthQueryApplicationHome<Schema, Program>),
    WorldAdmission {
        query: WorthQueryProgramApplicationRuntime<Schema, Program>,
        home: DirectoryHome,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        open_disposition: WorthQueryHomeOpenDisposition,
        active_head: worth_query_execution::facade::product::WorthQueryProductBranch,
        cause: ApplicationHomeCloseDenial,
    },
    QueryAdmission {
        query: WorthQueryProgramApplicationRuntime<Schema, Program>,
        home: DirectoryHome,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        open_disposition: WorthQueryHomeOpenDisposition,
        active_head: worth_query_execution::facade::product::WorthQueryProductBranch,
        cause: WorthQueryHomeRetirementDenial,
    },
    World {
        query: WorthQueryPreparedHomeQuery<Schema, Program>,
        home: WorldHomeCloseRefused<(), (), (), (), ()>,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        open_disposition: WorthQueryHomeOpenDisposition,
        active_head: worth_query_execution::facade::product::WorthQueryProductBranch,
    },
    QueryRetirement {
        query: WorthQueryRetiringHomeQuery<Schema, Program>,
        home: WorldSealedApplicationHome,
        cause: WorthQueryHomeRetirementDenial,
    },
    MaintenanceRetirement {
        query: WorthQueryRetiringHomeQuery<Schema, Program>,
        home: WorldSealedApplicationHome,
        cause: ApplicationHomeCloseDenial,
    },
    QueryReleased {
        cause: ApplicationHomeCloseDenial,
        posture: ApplicationHomeClosePosture,
        retired_query: Option<WorthQueryRetiredHomeQuery>,
    },
}
impl<Schema, Program> std::fmt::Debug for WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = f.debug_struct("WorthQueryApplicationHomeCloseRefused");
        output.field("reason", &self.detail());
        output.field("phase", &self.phase().label());
        if let CloseCustody::QueryReleased {
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
            ..
        } = self.custody.as_ref()
        {
            output.field("checkpoint", custody.checkpoint_result());
        }
        output.finish()
    }
}
impl<Schema: ApplicationSchema, Program: 'static> WorthQueryApplicationHome<Schema, Program> {
    pub fn close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryApplicationHomeClosed,
        WorthQueryApplicationHomeCloseRefused<Schema, Program>,
    > {
        // A Directory close releases every owned session and live subscription
        // before Query admission checks that no handle still holds the runtime.
        // It runs on the caller's thread, where thread-pinned subscriptions live.
        if self.directory.is_some() {
            self.owned.release_all();
        }
        worth_execution::run_with_execution_stack(move || self.close_on_execution_stack(request))
    }

    pub fn close_in_advancement(
        self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryApplicationHomeClosed,
        WorthQueryApplicationHomeCloseRefused<Schema, Program>,
    > {
        self.close(&phase.execution_request())
    }

    #[inline(never)]
    fn close_on_execution_stack(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryApplicationHomeClosed,
        WorthQueryApplicationHomeCloseRefused<Schema, Program>,
    > {
        let Self {
            query,
            directory,
            checkpoint_limits,
            open_disposition,
            active_head,
            owned,
            commit_watch,
            durable_historical_bases,
            memory_historical_bases,
            retained_branch_custody,
        } = self;
        let active_head = active_head
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(home) = directory.map(ErasedDirectoryHome::into_inner) else {
            return Err(WorthQueryApplicationHomeCloseRefused::from_custody(
                CloseCustody::MemoryLifecycleRequired(Self {
                    query,
                    directory: None,
                    checkpoint_limits,
                    open_disposition,
                    active_head: std::sync::Mutex::new(active_head),
                    owned,
                    commit_watch,
                    durable_historical_bases,
                    memory_historical_bases,
                    retained_branch_custody,
                }),
            ));
        };
        drop(retained_branch_custody);
        drop(durable_historical_bases);
        drop(memory_historical_bases);
        if let Err(cause) = home.preflight_world_close(request) {
            return Err(WorthQueryApplicationHomeCloseRefused::from_custody(
                CloseCustody::WorldAdmission {
                    query,
                    home,
                    checkpoint_limits,
                    open_disposition,
                    active_head,
                    cause,
                },
            ));
        }
        let query = match query.prepare_home_retirement(request) {
            Ok(query) => query,
            Err((query, cause)) => {
                return Err(WorthQueryApplicationHomeCloseRefused::from_custody(
                    CloseCustody::QueryAdmission {
                        query,
                        home,
                        checkpoint_limits,
                        open_disposition,
                        active_head,
                        cause,
                    },
                ))
            }
        };
        // No supplier callback occurs under World's final logical admission gates.
        match home.seal_world(request) {
            Ok(home) => retirement::finish_after_world(query, home, request),
            Err(home) => Err(WorthQueryApplicationHomeCloseRefused::from_custody(
                CloseCustody::World {
                    query,
                    home,
                    checkpoint_limits,
                    open_disposition,
                    active_head,
                },
            )),
        }
    }
}
impl<Schema, Program> WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    pub fn directory_denial(&self) -> Option<&ApplicationHomeCloseDenial> {
        match self.custody.as_ref() {
            CloseCustody::World { home, .. } => Some(home.reason()),
            CloseCustody::WorldAdmission { cause, .. } => Some(cause),
            CloseCustody::QueryReleased { cause, .. } => Some(cause),
            CloseCustody::MaintenanceRetirement { cause, .. } => Some(cause),
            _ => None,
        }
    }
    pub fn query_denial(&self) -> Option<&WorthQueryHomeRetirementDenial> {
        match self.custody.as_ref() {
            CloseCustody::QueryAdmission { cause, .. }
            | CloseCustody::QueryRetirement { cause, .. } => Some(cause),
            _ => None,
        }
    }
    pub fn detail(&self) -> String {
        match self.custody.as_ref() {
            CloseCustody::MemoryLifecycleRequired(_) => {
                "memory Home World lifecycle has not been readmitted".to_owned()
            }
            CloseCustody::World { home, .. } => format!("{:?}", home.reason()),
            CloseCustody::WorldAdmission { cause, .. } => format!("{cause:?}"),
            CloseCustody::QueryAdmission { cause, .. }
            | CloseCustody::QueryRetirement { cause, .. } => format!("{cause:?}"),
            CloseCustody::QueryReleased { cause, .. } => format!("{cause:?}"),
            CloseCustody::MaintenanceRetirement { cause, .. } => format!("{cause:?}"),
        }
    }
}
impl<Schema: ApplicationSchema, Program> WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    /// Only actual pre-seal custody may restore the original Query service.
    pub fn resume(self) -> Result<WorthQueryApplicationHome<Schema, Program>, Self> {
        match *self.custody {
            CloseCustody::MemoryLifecycleRequired(home) => Ok(home),
            CloseCustody::WorldAdmission {
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
                cause: _,
            } => Ok(WorthQueryApplicationHome {
                query,
                directory: Some(ErasedDirectoryHome::new(home)),
                checkpoint_limits,
                open_disposition,
                active_head: std::sync::Mutex::new(active_head),
                owned: std::sync::Arc::default(),
                commit_watch: std::sync::Arc::default(),
                durable_historical_bases: std::sync::OnceLock::new(),
                memory_historical_bases: std::sync::OnceLock::new(),
                retained_branch_custody: std::sync::Mutex::default(),
            }),
            CloseCustody::QueryAdmission {
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
                cause: _,
            } => Ok(WorthQueryApplicationHome {
                query,
                directory: Some(ErasedDirectoryHome::new(home)),
                checkpoint_limits,
                open_disposition,
                active_head: std::sync::Mutex::new(active_head),
                owned: std::sync::Arc::default(),
                commit_watch: std::sync::Arc::default(),
                durable_historical_bases: std::sync::OnceLock::new(),
                memory_historical_bases: std::sync::OnceLock::new(),
                retained_branch_custody: std::sync::Mutex::default(),
            }),
            CloseCustody::World {
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
            } => match home.resume() {
                Ok(home) => Ok(WorthQueryApplicationHome {
                    query: query.resume(),
                    directory: Some(ErasedDirectoryHome::new(home)),
                    checkpoint_limits,
                    open_disposition,
                    active_head: std::sync::Mutex::new(active_head),
                    owned: std::sync::Arc::default(),
                    commit_watch: std::sync::Arc::default(),
                    durable_historical_bases: std::sync::OnceLock::new(),
                    memory_historical_bases: std::sync::OnceLock::new(),
                    retained_branch_custody: std::sync::Mutex::default(),
                }),
                Err(home) => Err(Self::from_custody(CloseCustody::World {
                    query,
                    home,
                    checkpoint_limits,
                    open_disposition,
                    active_head,
                })),
            },
            custody => Err(Self::from_custody(custody)),
        }
    }
}

pub(super) fn classify_released_query<Schema, Program>(
    outcome: ApplicationHomeCloseOutcome,
    retired_query: Option<WorthQueryRetiredHomeQuery>,
) -> Result<WorthQueryApplicationHomeClosed, WorthQueryApplicationHomeCloseRefused<Schema, Program>>
{
    match outcome {
        outcome @ ApplicationHomeCloseOutcome::Closed { .. } => {
            Ok(WorthQueryApplicationHomeClosed {
                outcome,
                retired_query,
            })
        }
        ApplicationHomeCloseOutcome::CloseRefused { reason, posture } => Err(
            WorthQueryApplicationHomeCloseRefused::from_custody(CloseCustody::QueryReleased {
                cause: reason,
                posture,
                retired_query,
            }),
        ),
    }
}
