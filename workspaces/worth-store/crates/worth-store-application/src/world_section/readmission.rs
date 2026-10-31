//! Concrete Home custody cannot skip Native recovery or duplicate its World slot.
use worth_execution::ExecutionRequest;
use worth_proof::{AuthorityMarker, RelationalReadmitted, WorldReconciled};
use worth_runtime_world::facade::RuntimeWorldOwner;

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
    crate::ApplicationHomeOpenRefusal,
>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    native.consume(|mut home| {
        // Until Native's source-backed associated-effect admission is wired,
        // empty World rows are sufficient only under actual new-Home custody.
        // A recovered Native owner could hold an orphan associated effect.
        match home.context.origin {
            worth_store::physical_runtime::RecoveryStorePresence::ProvenAbsent => {}
            worth_store::physical_runtime::RecoveryStorePresence::ExistingArtifacts => {
                return Err(crate::ApplicationHomeOpenRefusal::World(
                    worth_runtime_world::facade::WorldReadmissionDenial::NativeReadmissionRequired,
                ));
            }
        }
        let slot = home
            .context
            .world_slot
            .take()
            .expect("the concrete Native stage owns one unclaimed World slot");
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
                Box::new(slot.into_medium()),
                request,
            )
            .map_err(crate::ApplicationHomeOpenRefusal::World)?;
        Ok(WorldReconciled::from_owner(
            WorldReadmittedApplicationHome { home, world },
        ))
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
