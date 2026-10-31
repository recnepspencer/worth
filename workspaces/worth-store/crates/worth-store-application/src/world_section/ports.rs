//! Existing World capabilities borrow the Home's sole lifecycle owner.
use worth_runtime_world::facade::*;

/// Attached World exposes publication, branch/history observation and recovery.
/// Standalone lifecycle authority remains with its actual Home.
///
/// ```compile_fail,E0599
/// use worth_store_application::ApplicationWorldPorts;
/// fn bypass_home(world: ApplicationWorldPorts<'_, (), (), (), (), ()>) {
///     world.lifecycle_port().close();
/// }
/// ```
///
/// ```
/// use worth_store_application::ApplicationWorldPorts;
/// fn publication(world: ApplicationWorldPorts<'_, (), (), (), (), ()>) {
///     let _ = world.publication_port();
/// }
/// ```
pub struct ApplicationWorldPorts<'home, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    owner: &'home RuntimeWorldOwner<D, I, E, Ctx, T>,
}
impl<'home, D, I, E, Ctx, T> ApplicationWorldPorts<'home, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn borrow(owner: &'home RuntimeWorldOwner<D, I, E, Ctx, T>) -> Self {
        Self { owner }
    }
    pub fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner.owner_identity()
    }
    pub fn lifecycle_observation(&self) -> RuntimeWorldOwnerLifecycleObservation {
        self.owner.lifecycle_observation()
    }
    pub fn publication_port(&self) -> RuntimeWorldPublicationPort<D, I, E, Ctx, T> {
        self.owner.publication_port()
    }
    pub fn branch_port(&self) -> RuntimeWorldBranchPort {
        self.owner.branch_port()
    }
    pub fn observation_port(&self) -> RuntimeWorldObservationPort {
        self.owner.observation_port()
    }
    pub fn inspection_port(&self) -> RuntimeWorldInspectionPort {
        self.owner.inspection_port()
    }
    pub fn recovery_port(&self) -> RuntimeWorldRecoveryPort {
        self.owner.recovery_port()
    }
}
