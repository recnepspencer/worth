//! The admitted producer's sealed old-input observation door.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::observation_reads::WorthQueryProducerObservationReadDenial,
    invariant_projection::operation_reader::BeforeDecisionReader,
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
        read: impl FnOnce(&mut BeforeDecisionReader<'_, '_, '_, S, O>) -> T,
    ) -> Result<T, WorthQueryProducerObservationReadDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        self.context_use
            .set(self.context_use.get().source_query_permission());
        self.reader.observe_predecessor(operation, read)
    }
    /// Read immutable DATA from the original admitted C, not predecessor INPUT.
    /// Fresh declared READ permission is independent of this handler's WRITE grant.
    pub fn original_observation<O, I, T>(
        &mut self,
        operation: ApplicationOperationRef<S, O, I>,
        read: impl FnOnce(&mut BeforeDecisionReader<'_, '_, '_, S, O>) -> T,
    ) -> Result<T, WorthQueryProducerObservationReadDenial>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
        I: 'static,
    {
        self.context_use
            .set(self.context_use.get().source_query_permission());
        self.reader.observe_original(operation, read)
    }
}
