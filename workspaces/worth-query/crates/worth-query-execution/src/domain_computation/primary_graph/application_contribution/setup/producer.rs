//! Producer installation owns its configured handler and input contract together.
use super::super::producer::input_capture::InputCaptureOwner;
use super::super::WorthQueryProducerInputCaptureHandler;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationProjection;
use worth_query_declaration::facade::application_program::WorthQueryProducerInputCaptureDefinition;

impl<Schema: ApplicationSchema> WorthQueryApplicationContributionSetup<'_, Schema> {
    pub fn producer<Binding>(
        &mut self,
        provider: Binding::Provider,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
        ProducerSourceValue<Schema, Binding>:
            WorthQueryApplicationProjection<Schema, ProducerSourceQuery<Schema, Binding>>,
    {
        self.register_producer::<Binding>(provider, None)
    }

    /// Install one original handler for both pure capture and normal production.
    /// The binding must declare this exact capture meaning and require a program.
    pub fn producer_with_input_capture<Binding, Capture, Handler>(
        &mut self,
        provider: Binding::Provider,
        handler: Handler,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
        Capture:
            WorthQueryProducerInputCaptureDefinition<Schema, OutputFamily = Binding::OutputFamily>,
        Handler: WorthQueryProducerInputCaptureHandler<Schema, Binding, Capture>,
        ProducerSourceValue<Schema, Binding>:
            WorthQueryApplicationProjection<Schema, ProducerSourceQuery<Schema, Binding>>,
    {
        self.handler::<Binding::Operation, Handler>(handler)?;
        self.register_producer::<Binding>(
            provider,
            Some(InputCaptureOwner::new::<Capture, Handler>()),
        )
    }

    fn register_producer<Binding>(
        &mut self,
        provider: Binding::Provider,
        input_capture: Option<InputCaptureOwner<Schema, Binding>>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
        ProducerSourceValue<Schema, Binding>:
            WorthQueryApplicationProjection<Schema, ProducerSourceQuery<Schema, Binding>>,
    {
        let source_query = self.require_owned_query::<ProducerSourceBinding<Schema, Binding>>()?;
        let mutation = self.require_owned_mutation::<Binding::Operation>()?;
        for requirement in Binding::REQUIRED_INVARIANTS {
            self.require_owned_invariant(*requirement)?;
        }
        self.producers.register::<Binding>(
            self.contribution.identity().as_str(),
            provider,
            mutation,
            self.installed_schema,
            source_query,
            input_capture,
        )
    }
}
