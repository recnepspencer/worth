//! The original root claim consumes the same disclosed source whose model is checked.
use super::*;
use crate::domain_computation::primary_graph::{
    application_installation::{
        WorthQueryAdmittedProgramOutput, WorthQueryProgramApplicationRuntime,
    },
    WorthQueryAdvancementPhase, WorthQueryApplicationOutputDemand,
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryHistoryRootOutputRead, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};
use worth_query_declaration::facade::{
    application_program::{
        ApplicationConnectionShape, ApplicationFixedOutputRoot, ApplicationOutputGraphShape,
        ApplicationProgramDefinition,
    },
    application_query::ApplicationQueryBinding,
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::ApplicationSchema;
mod dependent;
mod progression;
pub(in crate::domain_computation::primary_graph) use dependent::HistoryDependentSource;
pub use dependent::{WorthQueryHistoryDependentOutputRead, WorthQueryHistoryOutputDiscovery};
pub use progression::{
    WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryProgramOutputAdvance,
    WorthQuerySettledHistoryProgramOutput,
};

type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
type Demand<S, R> = <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
type Source<S,R> = <<Demand<S,R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source;
type Query<S, R> = <Source<S, R> as ApplicationQueryBinding<S>>::Query;
type Rows<S,R> = <<Source<S,R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// Original source-model correspondence issued only after the same disclosure
/// enters its actual root claim. It grants no READ, demand or navigation authority.
pub struct WorthQueryHistoryOutputSource {
    pub(super) profile: WorthQueryHistoryOutputProfile,
    pub(super) source: Arc<WorthQueryApplicationReadObservation>,
    pub(super) root_completion: Arc<crate::domain_computation::primary_graph::application_installation::OriginalProgramRootCompletion>,
}

/// History output custody has no conversion to an ordinary program demand.
/// Advancing it must supply a fresh history invocation for current control.
///
/// ```compile_fail,E0308
/// use worth_query_execution::{
///     facade::{application_installation::WorthQueryAdmittedProgramOutput,
///         application_contribution::WorthQueryApplicationOutputDemand},
///     publication_boundary::document_history::WorthQueryAdmittedHistoryProgramOutput,
/// };
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
/// fn ordinary<S, P, D>(_: &mut WorthQueryAdmittedProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {}
/// fn bypass<S, P, D>(history: &mut WorthQueryAdmittedHistoryProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {
///     ordinary(history);
/// }
/// ```
///
/// ```
/// use worth_query_execution::facade::{
///     application_installation::WorthQueryAdmittedProgramOutput,
///     application_contribution::WorthQueryApplicationOutputDemand,
/// };
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
/// fn ordinary<S, P, D>(_: &mut WorthQueryAdmittedProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {}
/// fn supported<S, P, D>(demand: &mut WorthQueryAdmittedProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {
///     ordinary(demand);
/// }
/// ```
pub struct WorthQueryAdmittedHistoryProgramOutput<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub(in crate::domain_computation::primary_graph) admitted:
        WorthQueryAdmittedProgramOutput<S, P, D>,
    pub(in crate::domain_computation::primary_graph) profile: WorthQueryHistoryOutputProfile,
    pub(in crate::domain_computation::primary_graph) original:
        Arc<WorthQueryApplicationReadObservation>,
}

#[derive(Debug)]
pub enum WorthQueryHistoryRootOutputAdmissionDenial {
    Authority(WorthQueryHistoryNavigationDenial),
    Demand(WorthQueryOutputDemandDenial),
}

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn admit_history_program_root_output<R>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        read: WorthQueryHistoryRootOutputRead<'_, '_, '_, Query<S, R>, Rows<S, R>>,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
    ) -> Result<
        (
            WorthQueryAdmittedHistoryProgramOutput<S, P, Demand<S, R>>,
            WorthQueryHistoryOutputSource,
        ),
        WorthQueryHistoryRootOutputAdmissionDenial,
    >
    where
        R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    {
        use WorthQueryHistoryRootOutputAdmissionDenial as Denial;
        let (authority, result, truth) = read.into_claim();
        let profile = authority.profile();
        let original = authority.original();
        let actual = truth
            .as_ref()
            .map_or(original.product_observation(), |truth| truth.observation());
        let source = result.into_admitted_disclosed().into_output_demand_source();
        let matches = profile.program_type
            == crate::equality_type_id::EqualityTypeId::of::<P>().into_external()
            && profile.root == crate::equality_type_id::EqualityTypeId::of::<R>().into_external()
            && profile.program == *self.installed_program().revision()
            && original
                .belongs_to_selected_occurrence(self.runtime(), original.product_observation())
            && actual.owner_identity() == original.product_observation().owner_identity()
            && actual.branch_identity() == original.product_observation().branch_identity()
            && actual.lifecycle_incarnation()
                == original.product_observation().lifecycle_incarnation()
            && source.result_set_observation().is_some_and(|set| {
                profile
                    .source_intent()
                    .matches_output_source(set.source(), actual)
            });
        if !matches {
            return Err(Denial::Demand(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "history output source must resolve the original full model at D",
            )));
        }
        let source = authority
            .confirm_read(phase, self.runtime(), authority.request(), source)
            .map_err(Denial::Authority)?;
        // The SAME linear disclosure is consumed here; no preflight re-query or
        // reconstructed source DTO can stand in for the actual root claim.
        let execution = phase
            .checked_execution_request_for(self.runtime().product_runtime())
            .map_err(|cause| {
                Denial::Demand(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
                    "history root claim phase",
                ))
            })?;
        let root_completion = crate::domain_computation::primary_graph::application_installation::OriginalProgramRootCompletion::reserve(&execution).map_err(Denial::Demand)?;
        let mut admitted = self
            .admit_program_root_output::<R>(phase, access, source, limits)
            .map_err(Denial::Demand)?;
        admitted.original_root_completion = Some(Arc::clone(&root_completion));
        Ok((
            WorthQueryAdmittedHistoryProgramOutput {
                admitted,
                profile: profile.clone(),
                original: Arc::clone(original),
            },
            WorthQueryHistoryOutputSource {
                profile: profile.clone(),
                source: Arc::clone(original),
                root_completion,
            },
        ))
    }
}
