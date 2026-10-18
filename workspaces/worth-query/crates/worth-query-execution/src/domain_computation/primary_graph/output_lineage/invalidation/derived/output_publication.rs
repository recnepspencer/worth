//! Performed source completion installed with the original native settlement.
use super::*;

impl SourceInvalidationOwner {
    /// Actual producer publication carries its native source partition beside
    /// the sealed settlement. Ordinary reestablishment registers no completion.
    pub(in crate::domain_computation::primary_graph) fn register_completed_settlement(
        &self,
        registration: SettlementRegistration,
        partition: Option<[u8; 32]>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), SettlementRegistrationStop> {
        let prepared = self.prepare_settlement_at_basis(
            registration,
            RegistrationReadBasis::RetainedReadAllowed,
            None,
            partition,
            admission,
        )?;
        let cleanup = prepared
            .install()
            .map_err(|stopped| SettlementRegistrationStop::Edit(stopped.reason()))?;
        drop(cleanup);
        Ok(())
    }
}
