//! Attach B to the original Source and World after its real binding.
use super::super::{
    opening, refusal::WorthQueryApplicationHomeOpenRefused as OldRefused, runtime::*,
};
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_execution::facade::{
    application_contribution::WorthQueryBootstrapAdvancementPhase,
    application_home_integration::ReadmittedExistingHomeNativeMeaning,
    application_installation::WorthQueryApplicationProgramRoots,
};
use worth_store_application::{ApplicationHomeCheckpointLimits, ApplicationHomeOpeningObservation};
pub(super) struct BoundHomeProgramTransition<B: ApplicationSchemaComposition, BP> {
    pub(super) after: ReadmittedExistingHomeNativeMeaning<B, BP>,
    pub(super) home: DirectoryHome,
    pub(super) bridge: worth_runtime_bridge::facade::RuntimeBridge,
}
impl<B, BP> BoundHomeProgramTransition<B, BP>
where
    B: ApplicationSchemaComposition,
    BP: ApplicationProgramDefinition<B> + 'static,
    BP::Outputs: ApplicationProgramOutputsShape<B> + WorthQueryApplicationProgramRoots<B>,
{
    #[inline(never)]
    pub(super) fn attach(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        checkpoint_limits: ApplicationHomeCheckpointLimits,
        observation: ApplicationHomeOpeningObservation,
    ) -> Result<WorthQueryApplicationHome<B, BP>, OldRefused<B, BP>> {
        let Self {
            after,
            mut home,
            bridge,
        } = self;
        let (source, _, _) = match home.query_composition_parts() {
            Ok(parts) => parts,
            Err(cause) => {
                return Err(OldRefused::TransitionComposition {
                    query: after,
                    home,
                    cause,
                })
            }
        };
        let (after, builder, bindings) =
            match after.prepare_existing_home_signal_program(phase, source, bridge) {
                Ok(parts) => parts,
                Err(query) => {
                    return Err(OldRefused::TransitionSignalProgramPreparation {
                        home,
                        query: Box::new(query),
                    })
                }
            };
        let conditional = match home.handoff_query_programme(builder, &phase.execution_request()) {
            Ok(conditional) => conditional,
            Err(handoff) => {
                return Err(OldRefused::TransitionProgrammeHandoff {
                    query: after,
                    home,
                    handoff,
                    _bindings: bindings,
                })
            }
        };
        let after = match after.attach_existing_home_authored_signal(conditional) {
            Ok(after) => after,
            Err(query) => return Err(OldRefused::ExistingSourceAttachment { query, home }),
        };
        drop(bindings);
        let (query, home) = opening::existing::attach_world(phase, home, after, checkpoint_limits)?;
        opening::service::finish_service(phase, query, home, checkpoint_limits, observation)
    }
}
