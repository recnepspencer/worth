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

/// World has actually closed and dropped. Native teardown is deliberately
/// delayed until the Host releases its Query/source/Bridge observations.
pub struct WorldSealedApplicationHome {
    home: crate::ReadmittedApplicationHome,
    composition: Option<super::attachment::ApplicationHomeBridgeComposition>,
}
impl WorldSealedApplicationHome {
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.home.home_identity()
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.home.incarnation()
    }
    pub fn world_close_report(&self) -> &worth_runtime_world::facade::RuntimeWorldCloseReport {
        self.home
            .context
            .world_close_report
            .as_ref()
            .expect("actual World close selected this phase")
    }
    pub fn finish_close(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> crate::ApplicationHomeCloseOutcome {
        // Existing after_world_close custody preserves RecoveryRequired even
        // when Native is still genuinely serving or external reads remain.
        crate::ApplicationHomeRecoveryCustody::after_world_close(self.home, self.composition)
            .try_release(request)
    }
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
    pub fn try_seal_world(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorldSealedApplicationHome, Self> {
        self.home.seal_world(request)
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
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCloseOutcome, WorldHomeCloseRefused<D, I, E, Ctx, T>> {
        self.seal_world(request)
            .map(|sealed| sealed.finish_close(request))
    }

    pub fn seal_world(
        mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorldSealedApplicationHome, WorldHomeCloseRefused<D, I, E, Ctx, T>> {
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
            let report = match self.world.lifecycle_port().close(request, &self.world) {
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
            composition,
        } = self;
        drop(world_cleanup);
        // Weak service ports and descriptive reports do not own this World.
        // Caller observations keep their own actual Native read custody, which
        // the unchanged Native close admission may still truthfully refuse.
        drop(world);
        Ok(WorldSealedApplicationHome { home, composition })
    }
}
