//! World closes through the same Home close/recovery lifecycle. Its actual
//! close report and actual owner travel into deferred retirement custody.
use super::WorldReadmittedApplicationHome;
use worth_execution::ExecutionRequest;
use worth_runtime_world::facade::RuntimeWorldOwnerLifecycleObservation;
mod admission;
mod bridge_retirement;
mod checkpoint;
pub(crate) use bridge_retirement::{
    HomeBridgeCloseCustody, PreparedHomeBridgeRetirement, RetiredHomeBridgeComposition,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorldHomeClosePosture {
    Serving,
    RecoveryRequired(RuntimeWorldOwnerLifecycleObservation),
}

/// World has sealed and moved into its actual retirement. Native teardown is
/// delayed until the Host releases its Query/source/Bridge observations.
pub struct WorldSealedApplicationHome {
    home: crate::ReadmittedApplicationHome,
    composition: Option<super::attachment::ApplicationHomeBridgeComposition>,
}
impl WorldSealedApplicationHome {
    #[doc(hidden)]
    pub fn retire_checkpoint_operation(
        &self,
        operation: crate::ApplicationHomeCheckpointOperation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::RetiredApplicationHomeCheckpointOperation,
        (
            crate::ApplicationHomeCheckpointOperation,
            crate::ApplicationHomeCloseDenial,
        ),
    > {
        operation.retire_for_home(&self.home.context.serving, request)
    }
    /// Closed Host retirement borrows the same Native owner; no raw handle or
    /// source escapes, and its Store-owned Arc remains alive throughout.
    #[doc(hidden)]
    pub fn with_query_retirement_owners<R>(
        &self,
        retire: impl FnOnce(
            &worth_relational::facade::runtime::RelationalRuntime,
            &std::sync::Arc<
                std::sync::RwLock<worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly>,
            >,
        ) -> R,
    ) -> R {
        retire(
            self.home.relational().runtime(),
            &self
                .composition
                .as_ref()
                .expect("Query Home owns its original composition")
                .conditional,
        )
    }
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.home.home_identity()
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.home.incarnation()
    }
    /// Exact released origin and occurrence image keys, retained until explicit Home reclaim.
    pub fn checkpoint_origin_retirement(
        &self,
    ) -> Option<&worth_runtime_world::facade::WorldCheckpointOriginRetirement> {
        self.home.context.world_origin_retirement.as_ref()
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
        if self.home.context.world_close_report.is_none() {
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
            if let Err(reason) = self.prepare_composite_close(request) {
                return Err(WorldHomeCloseRefused { home: self, reason });
            }
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
            match self.world.result_snapshot_cleanup_after_close(request) {
                Ok(task) => self.world_cleanup = Some(task),
                Err(cause) => {
                    return Err(WorldHomeCloseRefused {
                        home: self,
                        reason: crate::ApplicationHomeCloseDenial::WorldResultSnapshot(cause),
                    })
                }
            }
        }
        // The SAME owning task stays in this Home before borrowing it for
        // Native release. Refusal never fabricates a serving World posture.
        loop {
            match self.world_cleanup.as_mut().expect("retained exact result task")
                .advance(request, self.home.relational().runtime()) {
                Ok(worth_runtime_world::facade::RuntimeWorldResultSnapshotCleanupProgress::Advanced) => continue,
                Ok(worth_runtime_world::facade::RuntimeWorldResultSnapshotCleanupProgress::Complete(_)) => break,
                Err(cause) => return Err(WorldHomeCloseRefused {
                    home: self,
                    reason: crate::ApplicationHomeCloseDenial::WorldResultSnapshot(cause),
                }),
            }
        }
        if let Err(reason) = checkpoint::settle_before_world_drop(&mut self, request) {
            return Err(WorldHomeCloseRefused { home: self, reason });
        }
        if self.home.context.world_origin_retirement.is_none() {
            match self.world.retire_checkpoint_origin(request, self.home.relational().runtime()) {
                Ok(retired) => self.home.context.world_origin_retirement = Some(retired),
                Err(worth_runtime_world::facade::WorldCheckpointOriginRetirementDenial::NotInstalled) => {
                    // This owner never installed a World basis, so it has no origin obligation.
                }
                Err(cause) => return Err(WorldHomeCloseRefused {
                    home: self,
                    reason: crate::ApplicationHomeCloseDenial::WorldCheckpointOrigin(cause),
                }),
            }
        }
        let Self {
            mut home,
            world,
            world_cleanup,
            composition,
        } = self;
        let cleanup = world_cleanup.expect("same complete original result cleanup");
        let report = home
            .context
            .world_close_report
            .as_ref()
            .expect("actual closed World report");
        match world.into_home_retirement(report, cleanup) {
            Ok(original) => home.context.world_owner_retirement = Some(original),
            Err((world, cleanup, cause)) => {
                return Err(WorldHomeCloseRefused {
                    home: Self {
                        home,
                        world,
                        world_cleanup: Some(cleanup),
                        composition,
                    },
                    reason: crate::ApplicationHomeCloseDenial::WorldOwnerRetirement(cause),
                })
            }
        }
        Ok(WorldSealedApplicationHome { home, composition })
    }
}
