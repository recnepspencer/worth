//! Decision-phase definition preparation uses the operation's exact product.
use super::{CandidateWriter, DecisionReader};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::{
    application_contribution::WorthQueryApplicationConditionalBinding, HandlerExecutionDenial,
    WorthQueryAdmittedApplicationConditionalDefinition, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationConditionalDefinitionAdmissionDenial as AdmissionDenial,
    WorthQueryConditionalClockHandle, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryHostConditionalPredicateProvider,
};

pub(in crate::domain_computation::primary_graph) struct ConditionalDefinitionAdmission<'a, Schema> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: Option<&'a WorthQueryProductBranchLease>,
}
impl<'a, Schema> ConditionalDefinitionAdmission<'a, Schema> {
    pub(in crate::domain_computation::primary_graph) fn new(
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: Option<&'a WorthQueryProductBranchLease>,
    ) -> Self {
        Self {
            application,
            product,
        }
    }
}

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Prepare a replacement for an installed conditional definition at this
    /// operation's exact product. Carry the returned owner permit in the decision
    /// and pass it to `CandidateWriter::advance_conditional_definition`.
    /// `Conditional` is the contribution's installed binding, whose `Installed`
    /// value is its native clock handle. The provider derives from admitted input
    /// and decision reads; this method neither publishes nor restores values.
    pub fn prepare_conditional_definition<Conditional, Node, Clock, Provider>(
        &mut self,
        provider: Arc<Provider>,
    ) -> Result<WorthQueryAdmittedApplicationConditionalDefinition, HandlerExecutionDenial>
    where
        Conditional: WorthQueryApplicationConditionalBinding<
            Schema,
            Installed = WorthQueryConditionalClockHandle<Schema, Node, Clock>,
        >,
        Node: 'static,
        Clock: 'static,
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
    {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use.set(self.context_use.get().opaque_reader());
        let context = &self.conditional_definitions;
        let product = context.product.ok_or_else(|| {
            HandlerExecutionDenial::new(AdmissionDenial::MissingProductObservation)
        })?;
        let handle = context
            .application
            .installed_conditionals
            .binding::<Conditional>()
            .ok_or_else(|| {
                HandlerExecutionDenial::new(AdmissionDenial::ConditionalBindingUnavailable {
                    identity: Conditional::IDENTITY,
                })
            })?;
        context
            .application
            .admit_conditional_definition_on_product(product, &handle, provider)
            .map_err(HandlerExecutionDenial::new)
    }
}

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    CandidateWriter<'_, Schema, Binding>
{
    /// Publish the decision's admitted definition successor with this candidate's
    /// Relational effects in one World publication.
    pub fn advance_conditional_definition(
        &mut self,
        definition: WorthQueryAdmittedApplicationConditionalDefinition,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.candidate.advance_conditional_definition(definition)
    }
}
