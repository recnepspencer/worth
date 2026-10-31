//! World sealing precedes release of Query's actual Native/Bridge keepalives.
use super::runtime::*;
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_store_application::*;

pub struct WorthQueryApplicationHomeClosed {
    outcome: ApplicationHomeCloseOutcome,
}
impl WorthQueryApplicationHomeClosed {
    pub fn is_closed(&self) -> bool {
        matches!(self.outcome, ApplicationHomeCloseOutcome::Closed { .. })
    }
    pub(super) fn into_outcome(self) -> ApplicationHomeCloseOutcome {
        self.outcome
    }
    pub fn outcome(&self) -> &ApplicationHomeCloseOutcome {
        &self.outcome
    }
}

pub struct WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    custody: CloseCustody<Schema, Program>,
}
enum CloseCustody<Schema, Program> {
    MemoryLifecycleRequired(WorthQueryApplicationHome<Schema, Program>),
    World {
        query: WorthQueryProgramApplicationRuntime<Schema, Program>,
        home: WorldHomeCloseRefused<(), (), (), (), ()>,
    },
    QueryReleased {
        cause: ApplicationHomeCloseDenial,
        posture: ApplicationHomeClosePosture,
    },
}
impl<Schema, Program> std::fmt::Debug for WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WorthQueryApplicationHomeCloseRefused")
            .field(&self.detail())
            .finish()
    }
}
impl<Schema, Program> WorthQueryApplicationHome<Schema, Program> {
    pub fn close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryApplicationHomeClosed,
        WorthQueryApplicationHomeCloseRefused<Schema, Program>,
    > {
        let Self { query, directory } = self;
        let Some(home) = directory else {
            // Conditional-runtime close is not a World/Home close. Preserve the
            // actual memory owner until its lifecycle admission is implemented.
            return Err(WorthQueryApplicationHomeCloseRefused {
                custody: CloseCustody::MemoryLifecycleRequired(Self {
                    query,
                    directory: None,
                }),
            });
        };
        match home.seal_world(request) {
            Ok(sealed) => finish_after_world(query, sealed, request),
            Err(home) => Err(WorthQueryApplicationHomeCloseRefused {
                custody: CloseCustody::World { query, home },
            }),
        }
    }
}
impl<Schema, Program> WorthQueryApplicationHomeCloseRefused<Schema, Program> {
    pub fn directory_denial(&self) -> Option<&ApplicationHomeCloseDenial> {
        match &self.custody {
            CloseCustody::MemoryLifecycleRequired(_) => None,
            CloseCustody::World { home, .. } => Some(home.reason()),
            CloseCustody::QueryReleased { cause, .. } => Some(cause),
        }
    }

    pub fn detail(&self) -> String {
        match &self.custody {
            CloseCustody::MemoryLifecycleRequired(_) => {
                "memory Home World lifecycle has not been readmitted".to_owned()
            }
            CloseCustody::World { home, .. } => format!("{:?}", home.reason()),
            CloseCustody::QueryReleased { cause, .. } => format!("{cause:?}"),
        }
    }
    /// Only the actual pre-seal World posture may restore this Query service.
    pub fn resume(self) -> Result<WorthQueryApplicationHome<Schema, Program>, Self> {
        match self.custody {
            CloseCustody::MemoryLifecycleRequired(home) => Ok(home),
            CloseCustody::World { query, home } => match home.resume() {
                Ok(home) => Ok(WorthQueryApplicationHome {
                    query,
                    directory: Some(home),
                }),
                Err(home) => Err(Self {
                    custody: CloseCustody::World { query, home },
                }),
            },
            custody @ CloseCustody::QueryReleased { .. } => Err(Self { custody }),
        }
    }
    pub fn try_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryApplicationHomeClosed, Self> {
        match self.custody {
            CloseCustody::MemoryLifecycleRequired(home) => home.close(request),
            CloseCustody::World { query, home } => match home.try_seal_world(request) {
                Ok(sealed) => finish_after_world(query, sealed, request),
                Err(home) => Err(Self {
                    custody: CloseCustody::World { query, home },
                }),
            },
            CloseCustody::QueryReleased { cause: _, posture } => {
                let outcome = match posture {
                    ApplicationHomeClosePosture::RecoveryRequired(home) => {
                        home.try_release(request)
                    }
                    // Preserve even an advanced Native-serving payload as
                    // recovery-only; it never recreates a World or Query service.
                    ApplicationHomeClosePosture::Serving(home) => home.close(request),
                };
                classify_released_query(outcome)
            }
        }
    }
}
fn finish_after_world<Schema, Program>(
    query: WorthQueryProgramApplicationRuntime<Schema, Program>,
    home: WorldSealedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorthQueryApplicationHomeClosed, WorthQueryApplicationHomeCloseRefused<Schema, Program>>
{
    // seal_world completed the SAME actual World report's snapshot cleanup.
    // Drop internal Query/source/assembly clones before Native close admission.
    drop(query);
    classify_released_query(home.finish_close(request))
}
pub(super) fn classify_released_query<Schema, Program>(
    outcome: ApplicationHomeCloseOutcome,
) -> Result<WorthQueryApplicationHomeClosed, WorthQueryApplicationHomeCloseRefused<Schema, Program>>
{
    match outcome {
        outcome @ ApplicationHomeCloseOutcome::Closed { .. } => {
            Ok(WorthQueryApplicationHomeClosed { outcome })
        }
        ApplicationHomeCloseOutcome::CloseRefused { reason, posture } => {
            Err(WorthQueryApplicationHomeCloseRefused {
                custody: CloseCustody::QueryReleased {
                    cause: reason,
                    posture,
                },
            })
        }
    }
}
