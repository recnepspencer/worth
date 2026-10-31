//! World closes through the same Home close/recovery lifecycle. Its actual
//! close report travels into that custody after owner Drop releases its pins.
use super::WorldReadmittedApplicationHome;
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::RuntimeWorldOwnerLifecycleObservation;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorldHomeClosePosture {
    Serving,
    RecoveryRequired(RuntimeWorldOwnerLifecycleObservation),
}

pub struct WorldHomeCloseRefused<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    home: WorldReadmittedApplicationHome<D, I, E, Ctx, T>,
    reason: crate::ApplicationHomeCloseDenial,
}
impl<D, I, E, Ctx, T> WorldHomeCloseRefused<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn reason(&self) -> &crate::ApplicationHomeCloseDenial {
        &self.reason
    }
    pub fn posture(&self) -> WorldHomeClosePosture {
        let actual = self.home.world.lifecycle_observation();
        if matches!(
            self.reason,
            crate::ApplicationHomeCloseDenial::World(
                worth_runtime_world::facade::RuntimeWorldServiceDenial::OwnerUnavailable(_)
            )
        ) {
            return WorldHomeClosePosture::RecoveryRequired(actual);
        }
        match actual {
            RuntimeWorldOwnerLifecycleObservation::Open => WorldHomeClosePosture::Serving,
            RuntimeWorldOwnerLifecycleObservation::Closing
            | RuntimeWorldOwnerLifecycleObservation::Closed => {
                WorldHomeClosePosture::RecoveryRequired(actual)
            }
        }
    }
    pub fn resume(self) -> Result<WorldReadmittedApplicationHome<D, I, E, Ctx, T>, Self> {
        match self.posture() {
            WorldHomeClosePosture::Serving => Ok(self.home),
            WorldHomeClosePosture::RecoveryRequired(_) => Err(self),
        }
    }
    pub fn try_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCloseOutcome, Self> {
        self.home.close(request)
    }
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn close(
        mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCloseOutcome, WorldHomeCloseRefused<D, I, E, Ctx, T>> {
        if let Err(cause) = crate::close::posture::interrupted(request) {
            return Err(WorldHomeCloseRefused {
                home: self,
                reason: crate::ApplicationHomeCloseDenial::Execution(cause),
            });
        }
        // Stop and observe accepted work outside World's admission locks. The
        // actual World close then rechecks these SAME slots under its existing
        // operation gate, so another admission cannot race check and seal.
        self.home
            .context
            .world_publications
            .settle_for_close(&self.home.context.serving);
        if self.home.context.world_close_report.is_none() {
            let report = match self.world.lifecycle_port().close() {
                Ok(report) => report,
                Err(cause) => {
                    return Err(WorldHomeCloseRefused {
                        home: self,
                        reason: crate::ApplicationHomeCloseDenial::World(cause),
                    })
                }
            };
            self.home.context.world_close_report = Some(report);
        }
        if self.world_cleanup.is_none() {
            match self.world.result_snapshot_cleanup_after_close() {
                Ok(task) => self.world_cleanup = Some(task),
                Err(cause) => {
                    return Err(WorldHomeCloseRefused {
                        home: self,
                        reason: crate::ApplicationHomeCloseDenial::World(
                            worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(cause),
                        ),
                    })
                }
            }
        }
        // The SAME owning task stays in this Home before borrowing it for
        // Native release. Refusal never fabricates a serving World posture.
        if let Err(cause) = self
            .world_cleanup
            .as_mut()
            .expect("retained exact result task")
            .advance()
        {
            return Err(WorldHomeCloseRefused {
                home: self,
                reason: crate::ApplicationHomeCloseDenial::WorldResultSnapshot(cause),
            });
        }
        let Self {
            home,
            world,
            world_cleanup,
        } = self;
        drop(world_cleanup);
        // Weak service ports and descriptive reports do not own this World.
        // Caller observations keep their own actual Native read custody, which
        // the unchanged Native close admission may still truthfully refuse.
        drop(world);
        Ok(home.close(request))
    }
}
