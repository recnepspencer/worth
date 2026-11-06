//! A refused seal keeps the actual settled root and its original child authority.
use super::*;
impl<S, P, R> WorthQueryApplicationProgramOutputHandle<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S>
        + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
    RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) fn retain_root_fulfillment(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        if self.pending_root_fulfillment.is_some() || self.source_receipt.is_none() {
            return Ok(());
        }
        let binding = self
            .source_binding
            .as_ref()
            .expect("performed source has its installed recipe");
        let fact = (binding.fulfill)(
            phase,
            application,
            self.root.as_ref().expect("actual required root"),
            self.source_preparation
                .as_ref()
                .expect("actual performed-source custody"),
            self.source_receipt.as_ref().unwrap(),
            self.root_settlement.as_ref().expect("actual settled root"),
        )
        .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?;
        self.pending_root_fulfillment = Some(fact);
        Ok(())
    }
}
