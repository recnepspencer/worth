//! Original steps and guarded binding consume one planned transition.
use super::{
    planning::PlannedHomeProgramTransition, preparation::ReadmittedHomeProgramTransition,
    refusal::*, source_attachment::BoundHomeProgramTransition,
};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_contribution::WorthQueryBootstrapAdvancementPhase;
use worth_store_application::ApplicationHomeTransitionIntentOutcome;
use WorthQueryHomeProgramTransitionDenial as Denial;
use WorthQueryHomeProgramTransitionOpenRefused as Refused;
impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    PlannedHomeProgramTransition<A, AP, B, BP>
{
    #[inline(never)]
    pub(super) fn perform(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<BoundHomeProgramTransition<B, BP>, Refused<A, AP, B, BP>> {
        let Self {
            mut owners,
            binding,
            bridge,
            plan,
            reserved,
            resources,
        } = self;
        let request = phase.execution_request();
        let performed = (|| {
            owners
                .home
                .stage_home_program_transition_intent(reserved, binding, &request)
                .map_err(Denial::Intent)?;
            loop {
                request.consult().map_err(Denial::Request)?;
                match owners
                    .home
                    .advance_home_program_transition_intent(&request)
                    .map_err(Denial::Intent)?
                {
                    ApplicationHomeTransitionIntentOutcome::Committed => break,
                    ApplicationHomeTransitionIntentOutcome::Pending => std::thread::yield_now(),
                    outcome @ (ApplicationHomeTransitionIntentOutcome::Refused(_)
                    | ApplicationHomeTransitionIntentOutcome::Unresolved
                    | ApplicationHomeTransitionIntentOutcome::SourceMismatch) => {
                        return Err(Denial::IntentOutcome(outcome))
                    }
                }
            }
            super::publication::publish(
                &mut owners.home,
                &owners.target,
                plan,
                resources,
                &request,
            )?;
            let final_source = owners
                .home
                .world()
                .observation_port()
                .observe_root_product_branch()
                .map_err(Denial::Observation)?;
            let receipt = owners
                .home
                .finish_home_program_transition(
                    &final_source,
                    owners.target.native_contracts(),
                    &request,
                )
                .map_err(Denial::Binding)?;
            let installation = owners
                .home
                .home_program_transition_installation_binding()
                .map_err(Denial::Intent)?
                .ok_or(Denial::InvalidPlan)?;
            Ok((receipt, installation))
        })();
        let (receipt, installation) = match performed {
            Ok(parts) => parts,
            Err(cause) => return Err(owners.refused(cause)),
        };
        let ReadmittedHomeProgramTransition {
            before,
            target,
            home,
            source,
        } = *owners;
        let after = match target.finish_home_program_transition(
            home.relational().runtime(),
            receipt,
            installation,
        ) {
            Ok(after) => after,
            Err(successor) => return Err(Refused::promotion(before, successor, home)),
        };
        drop((before, source));
        Ok(BoundHomeProgramTransition {
            after,
            home,
            bridge,
        })
    }
}
