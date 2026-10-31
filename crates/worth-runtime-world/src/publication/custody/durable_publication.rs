use super::ActiveAttemptCustody;
use crate::branch::{
    ProductBranchObservation, ProductBranchReferenceCell, ProductBranchReferenceLoss,
};
use crate::durability::{
    WorldDurableArtifact, WorldDurableMediumBinding, WorldDurableMediumDenial,
    WorldDurablePublicationStop, WorldRelationalEvidence,
};
use std::sync::Arc;

impl ActiveAttemptCustody {
    pub(crate) fn cancel_unsubmitted_movement(&mut self) {
        self.lease_resources()
            .resources_mut()
            .durable_movement
            .take()
            .expect("same movement before journal submission")
            .proven_no_effect();
    }

    pub(crate) fn finalize_durable_occurrence(
        &mut self,
        evidence: WorldRelationalEvidence,
    ) -> Result<(Arc<WorldDurableMediumBinding>, WorldDurableArtifact), WorldDurableMediumDenial>
    {
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        let open = resources
            .durable_open
            .as_ref()
            .expect("durable prepared attempt");
        let binding = open.binding.clone();
        let occurrence = resources
            .durable_occurrence
            .as_mut()
            .expect("pre-effect backing admitted");
        let subject = occurrence
            .finalize(evidence, binding.binding().incarnation())?
            .clone();
        Ok((binding, subject))
    }

    pub(crate) fn admit_durable_movement(
        &mut self,
        cell: &ProductBranchReferenceCell,
        expected: &ProductBranchObservation,
        subject: WorldDurableArtifact,
    ) -> Result<(), ProductBranchReferenceLoss> {
        let permit = cell.begin_durable_movement(expected, subject)?;
        self.lease_resources().resources_mut().durable_movement = Some(permit);
        Ok(())
    }

    pub(crate) fn advance_durable_occurrence(
        &mut self,
        binding: &Arc<WorldDurableMediumBinding>,
        subject: &WorldDurableArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldDurablePublicationStop> {
        match binding.publish(subject, execution) {
            Ok(committed) => {
                self.lease_resources()
                    .resources_mut()
                    .durable_movement
                    .as_mut()
                    .expect("same cell movement installed before I/O")
                    .acknowledge(committed);
                Ok(())
            }
            Err(WorldDurablePublicationStop::Refused(cause)) => {
                self.lease_resources()
                    .resources_mut()
                    .durable_movement
                    .take()
                    .expect("same unacknowledged movement")
                    .proven_no_effect();
                Err(WorldDurablePublicationStop::Refused(cause))
            }
            Err(cause) => Err(cause),
        }
    }
}
