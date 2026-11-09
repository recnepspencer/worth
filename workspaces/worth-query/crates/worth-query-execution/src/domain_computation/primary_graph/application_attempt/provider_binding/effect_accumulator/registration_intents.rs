//! Append registration records without replacing the sealed producer program.
use super::*;
impl WorthQueryRegisteredProviderEffects {
    /// Appends the Primary Graph registration records without releasing or
    /// replacing the sealed provider-effect batch.
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) fn bind_registration_intents(
        self,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
        emitted_effect_count: u64,
        aftermath_causality: Option<
            &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
        dispatch_basis: crate::domain_computation::primary_graph::provider::dispatch_outbox::WorthQueryDispatchOutboxBasis<'_>,
    ) -> Result<
        (
            Self,
            Option<
                crate::domain_computation::application_aftermath::WorthQueryPendingDispatchOutbox,
            >,
        ),
        &'static str,
    > {
        let idempotency = dispatch_basis.idempotency;
        let outcome_identity = dispatch_basis.outcome_identity;
        let mutation_partition = self.mutation_partition;
        let (mut batch, receipt_appendix) = provider.bind_application_idempotency_intent(
            self.batch,
            idempotency,
            outcome_identity,
            emitted_effect_count,
            mutation_partition,
        );
        if let Some(causality) = aftermath_causality {
            batch = provider.bind_application_aftermath_causality_intent(
                batch,
                causality,
                outcome_identity,
                mutation_partition,
            );
        }
        let (batch, dispatch_outbox) =
            provider.bind_application_dispatch_outbox(batch, dispatch_basis, mutation_partition)?;
        Ok((
            Self {
                receipt_appendix: Some(receipt_appendix),
                body_external_emission: self.body_external_emission,
                content_attachments: self.content_attachments,
                expected_steps: self.expected_steps,
                batch,
                emissions: self.emissions,
                output_correspondence: self.output_correspondence,
                mutation_partition,
            },
            dispatch_outbox,
        ))
    }
}
