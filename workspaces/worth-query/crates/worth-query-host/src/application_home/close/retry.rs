//! Retry retained close custody without replaying preflight or Query service.
use super::*;

impl<Schema: ApplicationSchema, Program: 'static>
    WorthQueryApplicationHomeCloseRefused<Schema, Program>
{
    pub fn try_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeClosed, Self> {
        match self.custody {
            CloseCustody::MemoryLifecycleRequired(home) => home.close(request),
            CloseCustody::WorldAdmission {
                query,
                home,
                checkpoint_limits,
                cause: _,
            } => WorthQueryApplicationHome {
                query,
                directory: Some(home),
                checkpoint_limits,
            }
            .close(request),
            CloseCustody::QueryAdmission {
                query,
                home,
                checkpoint_limits,
                cause: _,
            } => WorthQueryApplicationHome {
                query,
                directory: Some(home),
                checkpoint_limits,
            }
            .close(request),
            CloseCustody::World {
                query,
                home,
                checkpoint_limits,
            } => match home.try_seal_world(request) {
                Ok(home) => retirement::finish_after_world(query, home, request),
                Err(home) => Err(Self {
                    custody: CloseCustody::World {
                        query,
                        home,
                        checkpoint_limits,
                    },
                }),
            },
            CloseCustody::QueryRetirement {
                query,
                home,
                cause: _,
            } => retirement::advance(query, home, request),
            CloseCustody::QueryReleased {
                cause: _,
                posture,
                retired_query,
            } => {
                let outcome = match posture {
                    ApplicationHomeClosePosture::RecoveryRequired(home) => {
                        home.try_release(request)
                    }
                    ApplicationHomeClosePosture::Serving(home) => home.close(request),
                };
                classify_released_query(outcome, retired_query)
            }
        }
    }
}
