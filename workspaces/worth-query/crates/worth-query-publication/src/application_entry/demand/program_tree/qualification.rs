//! Qualification is available only after native custody has actually closed.
use super::*;
use worth_query_execution::{
    facade::primary_graph::WorthQueryCompletedReadOutputs,
    publication_boundary::{
        discovery_currentness::WorthQueryRequiredSetCurrentnessDenial, program_publication_access,
    },
};

impl<S, P, R> WorthQueryClosedProgramOutputDelivery<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    /// Qualifies the original complete read tree at the caller's pinned basis.
    /// This checks fresh scope, selected typed root, all original settlements
    /// and discovery. It retains original source provenance and this exact
    /// output basis, without reopening native work or issuing mutation authority.
    /// Refusal borrows and preserves this already-closed carrier.
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
    /// use worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
    /// fn qualify<S,P,R>(closed: &WorthQueryClosedProgramOutputDelivery<S,P,R>, pinned: &WorthQueryApplicationRetainedRequest<'_, '_, '_, S>)
    ///     -> Result<WorthQueryCompletedReadOutputs, WorthQueryOperationDeliveryDenial>
    /// where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
    ///     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    ///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
    ///     let fact = closed.qualify_at(pinned)?;
    ///     let _ = (fact.source_commit(), fact.root_output_commit(), fact.output_commit(), fact.root_posture());
    ///     Ok(fact)
    /// }
    /// ```
    /// ```compile_fail,E0451
    /// use worth_query_publication::facade::application_entry::WorthQueryCompletedReadOutputs;
    /// use worth_query_execution::facade::primary_graph::{WorthQueryApplicationReadObservation, WorthQueryOutputSettlementPosture};
    /// use worth_runtime_world::facade::CompositeCommitIdentity;
    /// use std::sync::Arc;
    /// fn forge(source: Arc<WorthQueryApplicationReadObservation>, root: CompositeCommitIdentity,
    ///     posture: WorthQueryOutputSettlementPosture, output: Arc<WorthQueryApplicationReadObservation>) -> WorthQueryCompletedReadOutputs {
    ///     WorthQueryCompletedReadOutputs { source, root, posture, output, _memory: None }
    /// }
    /// ```
    /// ```compile_fail,E0599
    /// use worth_query_publication::facade::application_entry::WorthQueryCompletedReadOutputs;
    /// fn select(fact: &WorthQueryCompletedReadOutputs) { fact.select(); }
    /// ```
    pub fn qualify_at(
        &self,
        retained: &WorthQueryApplicationRetainedRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryCompletedReadOutputs, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        let request = retained.base_request();
        if !self.delivery.matches(&request) {
            return Err(Denial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                let selected = retained
                    .retained_observation()
                    .select_on(&phase, request.application)
                    .map_err(Denial::Observation)?;
                if selected.product().product_branch() != self.delivery.branch {
                    return Err(Denial::Currentness(
                        WorthQueryOutputCurrentnessDenial::ForeignBranch,
                    ));
                }
                let access = program_publication_access();
                self.delivery
                    .runtime
                    .validate_selected_output_root::<R>(
                        &phase,
                        &access,
                        &retained.retained_observation().retained,
                    )
                    .map_err(Denial::Program)?;
                let settlement = &self.delivery.settlement;
                selected
                    .qualify_completed_read_outputs(
                        &phase,
                        &access,
                        request.scope,
                        &self.delivery.source.retained,
                        settlement.root_settlement().retained_settlement(),
                        settlement.retained_settlements().skip(1),
                        settlement.dependencies.as_ref(),
                    )
                    .map_err(|cause| {
                        Denial::Currentness(match cause {
                            WorthQueryRequiredSetCurrentnessDenial::Source(cause) => {
                                WorthQueryOutputCurrentnessDenial::Source(cause)
                            }
                            WorthQueryRequiredSetCurrentnessDenial::Output(cause) => {
                                WorthQueryOutputCurrentnessDenial::Output(cause)
                            }
                        })
                    })
            })
            .map_err(Denial::Execution)?
    }
}
