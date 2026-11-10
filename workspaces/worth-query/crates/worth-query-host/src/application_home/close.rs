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

pub struct WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    custody: CloseCustody<Schema, Program>,
}
enum CloseCustody<Schema, Program> {
    MemoryLifecycleRequired(WorthQueryApplicationHome<Schema, Program>),
    WorldAdmission {
        query: WorthQueryProgramApplicationRuntime<Schema, Program>,
        home: DirectoryHome,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        cause: ApplicationHomeCloseDenial,
    },
    QueryAdmission {
        query: WorthQueryProgramApplicationRuntime<Schema, Program>,
        home: DirectoryHome,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        cause: WorthQueryHomeRetirementDenial,
    },
    World {
        query: WorthQueryPreparedHomeQuery<Schema, Program>,
        home: WorldHomeCloseRefused<(), (), (), (), ()>,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
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
        let phase = match &self.custody {
            CloseCustody::MemoryLifecycleRequired(_) => "memory-lifecycle",
            CloseCustody::WorldAdmission { .. } => "world-admission",
            CloseCustody::QueryAdmission { .. } => "query-admission",
            CloseCustody::World { .. } => "world-close",
            CloseCustody::QueryRetirement { .. } => "query-retirement",
            CloseCustody::MaintenanceRetirement { .. } => "maintenance-retirement",
            CloseCustody::QueryReleased { .. } => "physical-close",
        };
        output.field("phase", &phase);
        if let CloseCustody::QueryReleased {
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
            ..
        } = &self.custody
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
        worth_execution::run_with_execution_stack(move || self.close_on_execution_stack(request))
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
        } = self;
        let Some(home) = directory else {
            return Err(WorthQueryApplicationHomeCloseRefused {
                custody: CloseCustody::MemoryLifecycleRequired(Self {
                    query,
                    directory: None,
                    checkpoint_limits,
                }),
            });
        };
        if let Err(cause) = home.preflight_world_close(request) {
            return Err(WorthQueryApplicationHomeCloseRefused {
                custody: CloseCustody::WorldAdmission {
                    query,
                    home,
                    checkpoint_limits,
                    cause,
                },
            });
        }
        let query = match query.prepare_home_retirement(request) {
            Ok(query) => query,
            Err((query, cause)) => {
                return Err(WorthQueryApplicationHomeCloseRefused {
                    custody: CloseCustody::QueryAdmission {
                        query,
                        home,
                        checkpoint_limits,
                        cause,
                    },
                })
            }
        };
        // No supplier callback occurs under World's final logical admission gates.
        match home.seal_world(request) {
            Ok(home) => retirement::finish_after_world(query, home, request),
            Err(home) => Err(WorthQueryApplicationHomeCloseRefused {
                custody: CloseCustody::World {
                    query,
                    home,
                    checkpoint_limits,
                },
            }),
        }
    }
}
impl<Schema, Program> WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    pub fn directory_denial(&self) -> Option<&ApplicationHomeCloseDenial> {
        match &self.custody {
            CloseCustody::World { home, .. } => Some(home.reason()),
            CloseCustody::WorldAdmission { cause, .. } => Some(cause),
            CloseCustody::QueryReleased { cause, .. } => Some(cause),
            CloseCustody::MaintenanceRetirement { cause, .. } => Some(cause),
            _ => None,
        }
    }
    pub fn query_denial(&self) -> Option<&WorthQueryHomeRetirementDenial> {
        match &self.custody {
            CloseCustody::QueryAdmission { cause, .. }
            | CloseCustody::QueryRetirement { cause, .. } => Some(cause),
            _ => None,
        }
    }
    pub fn detail(&self) -> String {
        match &self.custody {
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
        match self.custody {
            CloseCustody::MemoryLifecycleRequired(home) => Ok(home),
            CloseCustody::WorldAdmission {
                query,
                home,
                checkpoint_limits,
                cause: _,
            } => Ok(WorthQueryApplicationHome {
                query,
                directory: Some(home),
                checkpoint_limits,
            }),
            CloseCustody::QueryAdmission {
                query,
                home,
                checkpoint_limits,
                cause: _,
            } => Ok(WorthQueryApplicationHome {
                query,
                directory: Some(home),
                checkpoint_limits,
            }),
            CloseCustody::World {
                query,
                home,
                checkpoint_limits,
            } => match home.resume() {
                Ok(home) => Ok(WorthQueryApplicationHome {
                    query: query.resume(),
                    directory: Some(home),
                    checkpoint_limits,
                }),
                Err(home) => Err(Self {
                    custody: CloseCustody::World {
                        query,
                        home,
                        checkpoint_limits,
                    },
                }),
            },
            custody => Err(Self { custody }),
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
        ApplicationHomeCloseOutcome::CloseRefused { reason, posture } => {
            Err(WorthQueryApplicationHomeCloseRefused {
                custody: CloseCustody::QueryReleased {
                    cause: reason,
                    posture,
                    retired_query,
                },
            })
        }
    }
}
