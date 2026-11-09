//! A recording root consumes the configured full source artifact at its exact origin.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputDemand, WorthQueryApplicationOutputDemandSource,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryProducerOutputFamily,
};
use worth_query_declaration::facade::application_program::ApplicationConnectionShape;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
type Demand<S, R> = <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
type Family<S, R> = <Demand<S, R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
type Binding<S, R> = <Family<S, R> as WorthQueryProducerOutputFamily<S>>::Source;
type Query<S, R> = <Binding<S, R> as ApplicationQueryBinding<S>>::Query;
type Value<S,R> = <<Binding<S,R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
impl WorthQueryHistoryRecordingOutputOrigin {
    /// The ordinary query has authorized its actual result. This compares the
    /// full owner-issued source and canonical parameters before producer contact.
    pub fn require_recording_root_source<S, P, R>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        index: usize,
        data: &Arc<WorthQueryApplicationReadObservation>,
        source: &WorthQueryApplicationOutputDemandSource<Query<S, R>, Value<S, R>>,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S>,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    {
        let profile = self.profile.tree_profiles().get(index).ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "history recording tree index",
            )
        })?;
        let subject = || {
            format!(
                "history recording source {:?}: {}",
                profile.source_intent().source_subject(),
                profile.recording_family()
            )
        };
        let mismatch = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                subject(),
            )
        };
        if !profile.matches_recording_root::<S, P, R>(application) {
            return Err(mismatch());
        }
        let current = application
            .runtime()
            .on_branch(self.candidate)
            .select(phase)
            .map_err(|cause| WorthQueryOutputDemandDenial::product_selection(cause, subject()))?;
        if data.branch_incarnation() != self.candidate.occurrence()
            || current.product().observation() != data.product_observation()
        {
            return Err(mismatch());
        }
        let binding = application
            .runtime()
            .installed_schema()
            .installed_query_binding::<Binding<S, R>>()
            .map_err(|_| mismatch())?;
        if !profile
            .source_intent()
            .matches_binding(&profile.recording_model(), &binding)
        {
            return Err(mismatch());
        }
        let execution = phase
            .checked_execution_request_for(&application.runtime().runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
                    subject(),
                )
            })?;
        for observed in source.observed_sources() {
            if !profile
                .source_intent()
                .matches_recording_source(&execution, observed, data.product_observation())
                .map_err(crate::domain_computation::primary_graph::output_reuse::read_denial)?
            {
                return Err(mismatch());
            }
        }
        Ok(())
    }
}
