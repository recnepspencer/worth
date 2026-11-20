//! Concrete Home custody cannot skip Native recovery or duplicate its World slot.
mod transition_catalog;
mod transition_completion;
pub use transition_completion::{
    ApplicationHomeTransitionBindingStageDenial, ApplicationHomeTransitionFinishDenial,
};
use worth_execution::ExecutionRequest;
use worth_proof::{AuthorityMarker, RelationalReadmitted, WorldReconciled};
use worth_runtime_world::facade::RuntimeWorldOwner;
#[cfg(feature = "certification-test-authority")]
mod controls_for_test;
mod query;
mod query_programme;
pub use query_programme::{
    ApplicationHomeProgrammeHandoffDenial, ApplicationHomeProgrammeHandoffRefused,
};
mod signal;
pub use signal::{
    prepare_query_home_source, readmit_query_signal, PreparedQueryHomeSource,
    SignalReadmittedApplicationHome,
};
mod refusal;
pub(in crate::world_section) mod standalone_signal;
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
    pub(super) program_source_owner: std::sync::Arc<worth_proof::InstalledBranchProgramSourceOwner>,
    pub(super) signal_checkpoint: worth_signal::facade::durability::SignalAuthoredCheckpointPort,
    pub(super) standalone_signal:
        Option<worth_signal::facade::durability::SignalDurableOwner<D, I, E, Ctx, T>>,
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
        let super::ApplicationWorldDefinition {
            world: configuration,
            signal: program,
            vocabulary,
            bridge,
            graph_role,
        } = definition;
        if let Err(cause) = request.consult() {
            return Err(WorldHomeReadmissionRefused {
                cause: crate::ApplicationHomeOpenRefusal::Execution(cause),
                home: Box::new(home),
                composition: None,
            });
        }
        let composition =
            match super::attachment::attach_bridge(&home.relational, bridge, graph_role) {
                Ok(composition) => composition,
                Err(cause) => {
                    return Err(WorldHomeReadmissionRefused {
                        cause,
                        home: Box::new(home),
                        composition: None,
                    })
                }
            };
        let signal = match standalone_signal::readmit(&home, program, vocabulary, request) {
            Ok(signal) => signal,
            Err(cause) => {
                return Err(WorldHomeReadmissionRefused {
                    cause,
                    home: Box::new(home),
                    composition: Some(composition),
                })
            }
        };
        let attachment =
            match worth_runtime_world::facade::RuntimeWorldAuthoredSignalAttachment::install(
                &signal, request,
            ) {
                Ok(attachment) => attachment,
                Err(cause) => {
                    return Err(WorldHomeReadmissionRefused {
                        cause: crate::ApplicationHomeOpenRefusal::SignalAttachment(Box::new(cause)),
                        home: Box::new(home),
                        composition: Some(composition),
                    })
                }
            };
        let builder = configuration.with_authored_signal(attachment);
        let bridge = composition.ordinary.runtime_world_correspondence_port();
        let signal_checkpoint = signal.checkpoint_port();
        match readmit_attached_world(&home, builder, bridge, request) {
            Ok(world) => Ok(finish_attachment(
                home,
                world,
                Some(composition),
                Some(signal),
                signal_checkpoint,
            )),
            Err(cause) => Err(WorldHomeReadmissionRefused {
                cause,
                home: Box::new(home),
                composition: Some(composition),
            }),
        }
    })
}

fn readmit_attached_world<D, I, E, Ctx, T>(
    home: &crate::ReadmittedApplicationHome,
    builder: worth_runtime_world::facade::RuntimeWorldAuthoredHomeBuilder<D, I, E, Ctx, T>,
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
    use crate::{ApplicationHomeOpeningEdge as Edge, ApplicationHomeOpeningStage as Stage};
    home.context
        .opening_observation
        .arrive_for_test(Stage::WorldReconciliation, Edge::Before);
    let result = if let Some(admission) = home.relational.world_fresh_source_admission() {
        builder.admit_fresh_world_home(
            home.relational.runtime().owner_component_services(),
            bridge,
            medium,
            super::checkpoint::readmission_source(&home.context, request).map_err(|cause| {
                crate::ApplicationHomeOpenRefusal::World(
                    worth_runtime_world::facade::WorldReadmissionDenial::Medium(cause),
                )
            })?,
            request,
            home.relational.runtime(),
            admission,
        )
    } else if let Some(recovered) = home.relational.world_recovery_authority() {
        builder.readmit_world_home(
            home.relational.runtime().owner_component_services(),
            bridge,
            &home
                .context
                .world_preparation
                .as_ref()
                .expect("existing original World preparation before Native replay")
                .source,
            request,
            home.relational.runtime(),
            recovered,
        )
    } else {
        return Err(crate::ApplicationHomeOpenRefusal::World(
            worth_runtime_world::facade::WorldReadmissionDenial::NativeReadmissionRequired,
        ));
    }
    .map_err(crate::ApplicationHomeOpenRefusal::World);
    if result.is_ok() {
        home.context
            .opening_observation
            .arrive_for_test(Stage::WorldReconciliation, Edge::After);
    }
    result
}
fn finish_attachment<D, I, E, Ctx, T>(
    mut home: crate::ReadmittedApplicationHome,
    world: RuntimeWorldOwner<D, I, E, Ctx, T>,
    composition: Option<super::attachment::ApplicationHomeBridgeComposition>,
    standalone_signal: Option<
        worth_signal::facade::durability::SignalDurableOwner<D, I, E, Ctx, T>,
    >,
    signal_checkpoint: worth_signal::facade::durability::SignalAuthoredCheckpointPort,
) -> WorldReconciled<WorldReadmittedApplicationHome<D, I, E, Ctx, T>>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if let Some(mut prepared) = home.context.world_preparation.take() {
        if let Some(native) = home.relational.world_recovery_authority() {
            prepared.complete(native);
        }
        drop(prepared);
    }
    let program_source_owner = home
        .context
        .world_slot
        .as_ref()
        .expect("same admitted slot")
        .program_source_owner();
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
        standalone_signal,
        signal_checkpoint,
        program_source_owner,
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
    /// Closed composition handoff: the program owner observes the selected
    /// branch revision before minting its exact source attestation.
    pub fn branch_program_source_owner(
        &self,
    ) -> std::sync::Arc<worth_proof::InstalledBranchProgramSourceOwner> {
        std::sync::Arc::clone(&self.program_source_owner)
    }
    pub fn world(&self) -> super::ApplicationWorldPorts<'_, D, I, E, Ctx, T> {
        super::ApplicationWorldPorts::borrow(&self.world)
    }
    pub fn relational(&self) -> &worth_store_relational::RelationalStoreSection {
        self.home.relational()
    }
    pub fn home_identity(&self) -> worth_foundational::facade::HomeIdentity {
        self.home.home_identity()
    }
    /// The current descriptor meaning on this same admitted Home. Reading it
    /// grants neither transition authority nor permission to rewrite history.
    pub fn binding(&self) -> &worth_foundational::facade::ApplicationHomeBinding {
        self.home.binding()
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
