//! Concrete Home custody cannot skip Native recovery or duplicate its World slot.
use worth_execution::ExecutionRequest;
use worth_proof::{AuthorityMarker, RelationalReadmitted, WorldReconciled};
use worth_runtime_world::facade::RuntimeWorldOwner;
mod controls_for_test;
mod refusal;
pub use refusal::WorldHomeReadmissionRefused;

pub struct WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) home: crate::ReadmittedApplicationHome,
    pub(super) world: RuntimeWorldOwner<D, I, E, Ctx, T>,
    pub(super) world_cleanup:
        Option<worth_runtime_world::facade::RuntimeWorldResultSnapshotCleanup>,
}
impl<D, I, E, Ctx, T> AuthorityMarker for WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
}

pub fn readmit_world<D, I, E, Ctx, T>(
    native: RelationalReadmitted<crate::ReadmittedApplicationHome>,
    definition: super::ApplicationWorldDefinition<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    WorldReconciled<WorldReadmittedApplicationHome<D, I, E, Ctx, T>>,
    WorldHomeReadmissionRefused,
>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    native.consume(|mut home| {
        let result = (|| {
            let native_recovery =
                home.relational
                    .world_recovery_authority()
                    .ok_or(crate::ApplicationHomeOpenRefusal::World(
                    worth_runtime_world::facade::WorldReadmissionDenial::NativeReadmissionRequired,
                ))?;
            let medium = home
                .context
                .world_slot
                .as_ref()
                .expect("the concrete Native stage owns one unclaimed World slot")
                .medium();
            let super::ApplicationWorldDefinition {
                world: builder,
                bridge,
                graph_role,
            } = definition;
            let bridge = super::attachment::attach_bridge(&home.relational, bridge, graph_role)?;
            let world = builder
                .readmit_world_home(
                    home.relational.runtime().owner_component_services(),
                    bridge,
                    medium,
                    request,
                    home.relational.runtime(),
                    native_recovery,
                )
                .map_err(crate::ApplicationHomeOpenRefusal::World)?;
            Ok(world)
        })();
        match result {
            Ok(world) => {
                // Only actual successful admission consumes the exclusive slot.
                drop(
                    home.context
                        .world_slot
                        .take()
                        .expect("same original World slot"),
                );
                Ok(WorldReconciled::from_owner(
                    WorldReadmittedApplicationHome {
                        home,
                        world,
                        world_cleanup: None,
                    },
                ))
            }
            Err(cause) => Err(WorldHomeReadmissionRefused {
                cause,
                home: Box::new(home),
            }),
        }
    })
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn world(&self) -> super::ApplicationWorldPorts<'_, D, I, E, Ctx, T> {
        super::ApplicationWorldPorts::borrow(&self.world)
    }
    pub fn relational(&self) -> &worth_store_relational::RelationalStoreSection {
        self.home.relational()
    }
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.home.home_identity()
    }
    pub fn incarnation(&self) -> worth_foundational::facade::HomeIncarnation {
        self.home.incarnation()
    }
    pub fn checkpoint(
        &self,
        _request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
    }
    pub fn recover_checkpoint(
        &self,
        _request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        Err(crate::ApplicationHomeCheckpointDenial::WorldSectionRequired)
    }
}
