//! The admitted producer's sealed old-input observation door.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::predecessor_reads::WorthQueryPredecessorReadDenial,
    invariant_projection::operation_reader::PredecessorDecisionReader,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationStructuredValueBinding,
};
impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> DecisionReader<'_, '_, '_, S, B> {
    /// Independently authorize the declared read operation at C, then project
    /// exactly the predecessor publication's consumed inputs. Unavailability
    /// permits full computation; admission/control failures veto publication.
    pub fn predecessor_observation<O, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, O, I>,
        read: impl FnOnce(&mut PredecessorDecisionReader<'_, '_, '_, S, O>) -> T,
    ) -> Result<T, WorthQueryPredecessorReadDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        self.context_use
            .set(self.context_use.get().source_query_permission());
        self.reader.observe_predecessor(operation, read)
    }
}
