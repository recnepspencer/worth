//! A mixed original records its planned Signal leg durably before Native's
//! effect, so a kill between the two owner effects can still finish the leg.
use super::{
    WorldArtifactInput, WorldDurableArtifact, WorldDurableMediumDenial as Denial,
    WorldDurableOpenCustody, WorldDurablePublicationStop as Stop, WorldOpenComponentIntent,
};
use worth_execution::ExecutionRequest;
use worth_runtime_bridge::facade::BridgePlannedDefinitionLeg;

impl WorldDurableOpenCustody {
    /// Whether this open's owner effects include Native before Signal, so a
    /// kill between them needs the planned leg to finish.
    pub(crate) fn plans_signal_after_native(&self) -> bool {
        self.subject.open_attempt().is_some_and(|open| {
            open.component_intent == WorldOpenComponentIntent::RelationalAndSignal
        })
    }
    /// Publish the leg for this committed mixed open and return only once it
    /// is committed. Any other answer leaves Native unbegun and the open retained.
    pub(crate) fn publish_planned_signal_leg(
        &self,
        leg: &BridgePlannedDefinitionLeg,
        execution: &ExecutionRequest<'_, '_>,
        stopped: &dyn Fn() -> bool,
    ) -> Result<(), Denial> {
        let open = self
            .subject
            .open_attempt()
            .expect("an open custody retains its original open");
        debug_assert_eq!(
            open.component_intent,
            WorldOpenComponentIntent::RelationalAndSignal
        );
        let subject = WorldDurableArtifact::from_input(
            WorldArtifactInput::PlannedSignalLeg {
                association: &open.association,
                custody: open.custody,
                leg,
            },
            execution,
        )?;
        loop {
            match self.binding.publish(&subject, execution) {
                Ok(_committed) => return Ok(()),
                Err(Stop::Pending | Stop::Busy) => {
                    execution.consult().map_err(Denial::Request)?;
                    if stopped() {
                        return Err(Denial::PublicationBusy);
                    }
                    std::thread::yield_now();
                }
                Err(Stop::Unresolved | Stop::EvidenceRequired) => {
                    return Err(Denial::PublicationUnresolved)
                }
                Err(Stop::Refused(cause)) => return Err(cause.into_cause()),
            }
        }
    }
}
