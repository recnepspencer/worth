//! The original tree supplies its endpoint; the caller supplies fresh scope.
use super::*;
use worth_query_execution::{
    facade::primary_graph::WorthQueryCompletedOriginalReadOutputs,
    publication_boundary::{
        discovery_currentness::WorthQueryRequiredSetCurrentnessDenial,
        original_read_outputs::WorthQueryOriginalReadOutputDenial as NativeDenial,
        program_publication_access,
    },
};

impl<S, P, R> WorthQueryClosedProgramOutputDelivery<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    /// Qualifies the exact endpoint connected to the original read source by
    /// canonical publications retained by this complete tree. Receiptless reuse
    /// may leave the endpoint at the original source. Later unrelated movement
    /// is never captured as a replacement endpoint. This is retrospective
    /// qualification, not current-head selection or permission to adopt it.
    /// Refusal preserves this carrier and creates no producer or cleanup work.
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
    /// use worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
    /// fn qualify<S,P,R>(closed: &WorthQueryClosedProgramOutputDelivery<S,P,R>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
    ///     -> Result<WorthQueryCompletedOriginalReadOutputs, WorthQueryOperationDeliveryDenial>
    /// where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
    ///     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    ///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
    ///     let fact = closed.qualify_original(request)?;
    ///     let _ = (fact.source_commit(), fact.root_output_commit(), fact.output_commit(), fact.root_posture());
    ///     Ok(fact)
    /// }
    /// ```
    /// ```compile_fail,E0451
    /// use worth_query_publication::facade::application_entry::WorthQueryCompletedOriginalReadOutputs;
    /// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationReadObservation, WorthQueryOutputSettlementPosture};
    /// use std::sync::Arc;
    /// fn forge(source: Arc<WorthQueryApplicationReadObservation>, root: Arc<WorthQueryApplicationReadObservation>,
    ///     posture: WorthQueryOutputSettlementPosture, output: Arc<WorthQueryApplicationReadObservation>) -> WorthQueryCompletedOriginalReadOutputs {
    ///     WorthQueryCompletedOriginalReadOutputs { source, root, posture, output, _memory: None }
    /// }
    /// ```
    /// ```compile_fail,E0277
    /// use worth_query_publication::facade::application_entry::{WorthQueryCompletedReadOutputs, WorthQueryCompletedOriginalReadOutputs};
    /// fn cannot_upgrade(fact: WorthQueryCompletedReadOutputs) {
    ///     let _: WorthQueryCompletedOriginalReadOutputs = fact.into();
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use worth_query_publication::facade::application_entry::WorthQueryCompletedOriginalReadOutputs;
    /// fn cannot_select(fact: &WorthQueryCompletedOriginalReadOutputs) { fact.select(); }
    /// ```
    pub fn qualify_original(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryCompletedOriginalReadOutputs, WorthQueryOperationDeliveryDenial> {
        if !self.delivery.matches(request) {
            return Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.qualify_original_in_advancement(&phase, request)
            })
            .map_err(WorthQueryOperationDeliveryDenial::Execution)?
    }

    pub(in crate::application_entry) fn qualify_original_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryCompletedOriginalReadOutputs, WorthQueryOperationDeliveryDenial> {
        if !self.delivery.matches(request) {
            return Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch);
        }
        let settlement = &self.delivery.settlement;
        self.delivery
            .runtime
            .qualify_original_read_outputs::<R>(
                phase,
                &program_publication_access(),
                request.scope,
                &self.delivery.source.retained,
                settlement.root_settlement().retained_settlement(),
                settlement.retained_settlements().skip(1),
                settlement.dependencies.as_ref(),
            )
            .map_err(denial)
    }
}

fn denial(cause: NativeDenial) -> WorthQueryOperationDeliveryDenial {
    use WorthQueryOperationDeliveryDenial as Denial;
    match cause {
        NativeDenial::Observation(cause) => Denial::Observation(cause),
        NativeDenial::Program(cause) => Denial::Program(cause),
        NativeDenial::Output(cause) => {
            Denial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause))
        }
        NativeDenial::Currentness(WorthQueryRequiredSetCurrentnessDenial::Source(cause)) => {
            Denial::Currentness(WorthQueryOutputCurrentnessDenial::Source(cause))
        }
        NativeDenial::Currentness(WorthQueryRequiredSetCurrentnessDenial::Output(cause)) => {
            Denial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause))
        }
    }
}
