//! Concrete Home custody cannot skip Native recovery or duplicate its World slot.
use worth_execution::ExecutionRequest;
use worth_proof::{AuthorityMarker, RelationalReadmitted, WorldReconciled};
use worth_runtime_world::facade::RuntimeWorldOwner;
#[cfg(feature = "certification-test-authority")]
mod controls_for_test;
mod query;
mod refusal;
mod transition;
pub use query::{readmit_query_world, ApplicationHomeQueryCompositionDenial};
pub use refusal::WorldHomeReadmissionRefused;
pub use transition::{ApplicationHomeOwnerAdmissionRefused, NoPendingApplicationHomeTransition};

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
    let medium = home
        .context
        .world_slot
        .as_ref()
        .expect("one original exclusive World slot")
        .medium();
    let source = super::checkpoint::readmission_source(home, request).map_err(|cause| {
        crate::ApplicationHomeOpenRefusal::World(
            worth_runtime_world::facade::WorldReadmissionDenial::Medium(cause),
        )
    })?;
    if let Some(admission) = home.relational.world_fresh_source_admission() {
        builder.admit_fresh_world_home(
            home.relational.runtime().owner_component_services(),
            bridge,
            medium,
            source,
            request,
            home.relational.runtime(),
            admission,
        )
    } else if let Some(recovered) = home.relational.world_recovery_authority() {
        builder.readmit_world_home(
            home.relational.runtime().owner_component_services(),
            bridge,
            medium,
            source,
            request,
            home.relational.runtime(),
            recovered,
        )
    } else {
        return Err(crate::ApplicationHomeOpenRefusal::World(
            worth_runtime_world::facade::WorldReadmissionDenial::NativeReadmissionRequired,
        ));
    }
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
    // Both owners consumed this SAME initial image and tails. Any required
    // Native member route owns its actual opaque read share independently.
    // Release only this completed context share, outside admission locks.
    drop(home.context.selected_home_source.take());
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
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        self.checkpoint_operation(Default::default())?
            .checkpoint(request)
    }
    pub fn recover_checkpoint(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::ApplicationHomeCheckpoint, crate::ApplicationHomeCheckpointDenial> {
        self.checkpoint_operation(Default::default())?
            .recover_checkpoint(request)
    }
}

#[cfg(test)]
#[path = "../tests/world_section/readmission.rs"]
mod tests;
