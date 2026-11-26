//! Retry retained close custody without replaying preflight or Query service.
use super::*;

impl<Schema: ApplicationSchema, Program: 'static>
    WorthQueryApplicationHomeCloseRefused<Schema, Program>
{
    pub fn try_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeClosed, Self> {
        worth_execution::run_with_execution_stack(move || {
            self.try_close_on_execution_stack(request)
        })
    }

    #[inline(never)]
    fn try_close_on_execution_stack(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeClosed, Self> {
        match *self.custody {
            CloseCustody::MemoryLifecycleRequired(home) => Self::retry_home_close(home, request),
            CloseCustody::WorldAdmission {
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
                cause: _,
            }
            | CloseCustody::QueryAdmission {
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
                cause: _,
            } => Self::retry_home_close(
                WorthQueryApplicationHome {
                    query,
                    directory: Some(ErasedDirectoryHome::new(home)),
                    checkpoint_limits,
                    open_disposition,
                    active_head: std::sync::Mutex::new(active_head),
                    owned: std::sync::Arc::default(),
                    commit_watch: std::sync::Arc::default(),
                },
                request,
            ),
            CloseCustody::World {
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
            } => Self::retry_world_close(
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
                request,
            ),
            CloseCustody::QueryRetirement {
                query,
                home,
                cause: _,
            }
            | CloseCustody::MaintenanceRetirement {
                query,
                home,
                cause: _,
            } => retirement::advance(query, home, request),
            CloseCustody::QueryReleased {
                cause: _,
                posture,
                retired_query,
            } => Self::retry_released_query_close(posture, retired_query, request),
        }
    }

    #[inline(never)]
    fn retry_home_close(
        home: WorthQueryApplicationHome<Schema, Program>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeClosed, Self> {
        home.close(request)
    }

    #[inline(never)]
    fn retry_world_close(
        query: WorthQueryPreparedHomeQuery<Schema, Program>,
        home: WorldHomeCloseRefused<(), (), (), (), ()>,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        open_disposition: WorthQueryHomeOpenDisposition,
        active_head: worth_query_execution::facade::product::WorthQueryProductBranch,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeClosed, Self> {
        match home.try_seal_world(request) {
            Ok(home) => retirement::finish_after_world(query, home, request),
            Err(home) => Err(Self::from_custody(CloseCustody::World {
                query,
                home,
                checkpoint_limits,
                open_disposition,
                active_head,
            })),
        }
    }

    #[inline(never)]
    fn retry_released_query_close(
        posture: ApplicationHomeClosePosture,
        retired_query: Option<WorthQueryRetiredHomeQuery>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeClosed, Self> {
        let outcome = match posture {
            ApplicationHomeClosePosture::RecoveryRequired(home) => home.try_release(request),
            ApplicationHomeClosePosture::Serving(home) => home.close(request),
        };
        classify_released_query(outcome, retired_query)
    }
}
