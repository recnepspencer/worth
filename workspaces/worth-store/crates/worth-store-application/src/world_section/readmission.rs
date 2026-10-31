//! Concrete Home custody cannot skip Native recovery or duplicate its World slot.
use worth_execution::ExecutionRequest;
use worth_proof::{AuthorityMarker, RelationalReadmitted, WorldReconciled};
use worth_runtime_world::facade::RuntimeWorldOwner;
#[cfg(feature = "certification-test-authority")]
mod controls_for_test;
mod query;
mod refusal;
pub use query::{readmit_query_world, ApplicationHomeQueryCompositionDenial};
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
    pub(super) composition: Option<super::attachment::ApplicationHomeBridgeComposition>,
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
    native.consume(|home| {
        let result = (|| {
            request
                .consult()
                .map_err(crate::ApplicationHomeOpenRefusal::Execution)?;
            let super::ApplicationWorldDefinition {
                world: builder,
                bridge,
                graph_role,
            } = definition;
            let bridge = super::attachment::attach_bridge(&home.relational, bridge, graph_role)?;
            readmit_attached_world(&home, builder, bridge, request)
        })();
        match result {
            Ok(world) => Ok(finish_attachment(home, world, None)),
            Err(cause) => Err(WorldHomeReadmissionRefused {
                cause,
                home: Box::new(home),
                composition: None,
            }),
        }
    })
}

fn readmit_attached_world<D, I, E, Ctx, T>(
    home: &crate::ReadmittedApplicationHome,
    builder: worth_runtime_world::facade::RuntimeWorldHomeBuilder<D, I, E, Ctx, T>,
    bridge: worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RuntimeWorldOwner<D, I, E, Ctx, T>, crate::ApplicationHomeOpenRefusal>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let native_recovery = home.relational.world_recovery_authority().ok_or(
        crate::ApplicationHomeOpenRefusal::World(
            worth_runtime_world::facade::WorldReadmissionDenial::NativeReadmissionRequired,
        ),
    )?;
    let medium = home
        .context
        .world_slot
        .as_ref()
        .expect("one original exclusive World slot")
        .medium();
    builder
        .readmit_world_home(
            home.relational.runtime().owner_component_services(),
            bridge,
            medium,
            request,
            home.relational.runtime(),
            native_recovery,
        )
        .map_err(crate::ApplicationHomeOpenRefusal::World)
}
fn finish_attachment<D, I, E, Ctx, T>(
    mut home: crate::ReadmittedApplicationHome,
    world: RuntimeWorldOwner<D, I, E, Ctx, T>,
    composition: Option<super::attachment::ApplicationHomeBridgeComposition>,
) -> WorldReconciled<WorldReadmittedApplicationHome<D, I, E, Ctx, T>>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    drop(
        home.context
            .world_slot
            .take()
            .expect("only actual admission consumes the same slot"),
    );
    WorldReconciled::from_owner(WorldReadmittedApplicationHome {
        home,
        world,
        world_cleanup: None,
        composition,
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

#[cfg(test)]
#[path = "../tests/world_section/readmission.rs"]
mod tests;
