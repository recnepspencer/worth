//! Decision-phase definition preparation uses the operation's exact product.
use std::sync::Arc;
use super::{CandidateWriter, DecisionReader};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::conditional_operation::installation::clock_handle::WorthQueryConditionalClockHandle;
use crate::domain_computation::primary_graph::product_operation::conditional_definition::application_change::WorthQueryAdmittedApplicationConditionalDefinitionRuntimeExt as _;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryAdmittedApplicationConditionalDefinition,
    WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationConditionalDefinitionAdmissionDenial as AdmissionDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryAuthoredHostConditionalPredicateProvider,
};

pub struct ConditionalDefinitionAdmission<'a, Schema> {
    pub(crate) application: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(crate) product: Option<&'a WorthQueryProductBranchLease>,
}

impl<'a, Schema> ConditionalDefinitionAdmission<'a, Schema> {
    pub fn new(
        application: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: Option<&'a WorthQueryProductBranchLease>,
    ) -> Self {
        Self {
            application,
            product,
        }
    }

    pub fn product(&self) -> Option<&'a WorthQueryProductBranchLease> {
        self.product
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
        Conditional: 'static,
        Node: 'static,
        Clock: 'static,
        Provider: WorthQueryAuthoredHostConditionalPredicateProvider<Node>,
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
            .entry_value_by_type(
                TypeId::of::<Conditional>(),
                TypeId::of::<WorthQueryConditionalClockHandle<Schema, Node, Clock>>(),
            )
            .and_then(|value| {
                value
                    .downcast::<WorthQueryConditionalClockHandle<Schema, Node, Clock>>()
                    .ok()
            })
            .ok_or_else(|| {
                HandlerExecutionDenial::new(AdmissionDenial::ConditionalBindingUnavailable {
                    identity: std::any::type_name::<Conditional>(),
                })
            })?;
        let port = context
            .application
            .conditional_admission_port
            .upgrade()
            .ok_or_else(|| {
                HandlerExecutionDenial::new(AdmissionDenial::ForeignConditionalOperation)
            })?;
        let bridge_root = context.application.runtime.bridge.conditional_operations();
        let (predecessor, authored) = port
            .resolve_predecessor(&bridge_root, product, handle.binding_identity(), handle.lease())
            .map_err(HandlerExecutionDenial::new)?;
        let request = if authored {
            let source =
                worth_query_installation::facade::WorthQueryAuthoredHostPredicateBinding::from_provider(
                    provider,
                );
            let provider =
                crate::domain_computation::primary_graph::conditional_operation::authored_predicate::QueryAuthoredPredicateProvider::new(
                    source,
                    Arc::from(predecessor.contract().identity()),
                    Arc::clone(handle.node_authority()),
                );
            match predecessor.contract().condition() {
                worth_runtime_bridge::facade::BridgeConditionalCondition::RuntimePredicate => {
                    predecessor.successor_with_authored_condition_provider(provider)
                }
                worth_runtime_bridge::facade::BridgeConditionalCondition::Always
                | worth_runtime_bridge::facade::BridgeConditionalCondition::AspectFiltered
                | worth_runtime_bridge::facade::BridgeConditionalCondition::DeltaThreshold(_)
                | worth_runtime_bridge::facade::BridgeConditionalCondition::OnDemand
                | worth_runtime_bridge::facade::BridgeConditionalCondition::TemporalWake => {
                    predecessor.successor_with_authored_wake_provider(provider)
                }
            }
        } else {
            predecessor.successor_with_wake_provider(
                crate::domain_computation::primary_graph::conditional_operation::predicate_observation::QueryTemporalPredicateProvider::<Node, Provider>::new(
                    provider,
                    Arc::clone(handle.node_authority()),
                ),
            )
        };
        Ok(WorthQueryAdmittedApplicationConditionalDefinition::new(
            context.application,
            product,
            *handle.binding_identity_digest(),
            predecessor,
            request,
        ))
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

mod court_doors_for_test;
