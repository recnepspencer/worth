//! Reverse capture stays bound to the original preserved output, not inverse role spelling.
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as Posture;
use worth_query_installation::facade::{InstalledCorrectionMechanism, InstalledPreImageDemand};

impl WorthQueryPreparedCorrectionSource {
    pub(in crate::domain_computation::primary_graph) fn changed_field_capture(
        &self,
    ) -> Result<Option<(InstalledPreImageDemand, EntityId)>, ()> {
        if self.lineage.role() != WorthQueryAftermathCausalRole::RecordedReverse {
            return Ok(None);
        }
        let original = self.custody.occurrence();
        let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) = original
            .aftermath()
            .and_then(|contract| contract.mechanism())
        else {
            return Ok(None);
        };
        let demand = inverse.preimage_demand();
        let Some(output) = demand.output_fields().filter(|output| output.capture() == worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture::ChangedPreserved) else {
            return Ok(None);
        };
        let mut roles = original.correspondence().native_witness_roles().filter(
            |(name, posture, entity, _)| {
                *name == output.role()
                    && *posture == Posture::Preserve
                    && *entity == output.entity()
            },
        );
        let (_, _, _, target) = roles.next().ok_or(())?;
        if roles.next().is_some() {
            return Err(());
        }
        Ok(Some((demand.clone(), target)))
    }
}
