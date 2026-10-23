//! Private replay lookup discloses the actual performed inverse input without resuming effects.
use super::intent::WorthQueryCorrectionIntent;
use super::*;
use crate::domain_computation::primary_graph::provider::CorrectionIdempotencyObservation as Observed;
use worth_query_declaration::facade::{
    application_aftermath::ApplicationInverseBinding,
    application_operation::{ApplicationEncodedInput, ApplicationMutationBinding},
};

pub enum WorthQueryCorrectionInputSelection<
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
> {
    FirstUse(WorthQueryCorrectionIntent),
    Replay { intent: WorthQueryCorrectionIntent,
        input: ApplicationEncodedInput<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding> },
    IntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}

pub(super) fn select_input<Schema: ApplicationSchema, C: ApplicationCorrectionBinding<Schema>>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    key: &WorthQueryCorrectionCommandKey,
    maximum_history: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
) -> Result<WorthQueryCorrectionInputSelection<Schema, C>, WorthQueryCorrectionDerivationDenial> {
    disclosure::disclose_correction_source::<Schema, C>(
        target,
        phase,
        principal,
        request,
        source,
        maximum_history,
        maximum_results,
    )?;
    let intent =
        WorthQueryCorrectionIntent::from_disclosed_source::<Schema, C>(target, source, key);
    use WorthQueryCorrectionInputSelection as Selection;
    Ok(
        match target
            .application()
            .primary_provider
            .observe_correction_key(target.product().observation(), &key.identity())
        {
            Observed::Unseen => Selection::FirstUse(intent),
            Observed::Completed(binding, committed) => {
                if !intent.matches(&binding)
                    || !binding
                        .is_for_mutation_binding(ApplicationInverseBinding::<Schema, C>::IDENTITY)
                {
                    return Ok(Selection::IntentDrift);
                }
                let input = committed.commit_evidence().original_authority().retained_governed_input_carrier()
                .and_then(|retained| retained.encoded_input::<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>())
                .ok_or(WorthQueryCorrectionDerivationDenial::SemanticMaterialUnavailable)?;
                if input.identity() != binding.intent_identity() {
                    return Err(WorthQueryCorrectionDerivationDenial::UnsupportedSourceMeaning);
                }
                Selection::Replay {
                    intent: intent.with_recorded_definition(&binding),
                    input: input.clone(),
                }
            }
            Observed::Pending(Some(binding))
                if binding.key_identity() == &key.identity() && !intent.matches(&binding) =>
            {
                Selection::IntentDrift
            }
            Observed::Pending(_) => Selection::PublicationPending,
            Observed::Unpublished(binding, handle) if intent.matches(&binding) => {
                Selection::ProductUnpublished(handle)
            }
            Observed::Unpublished(_, _) => Selection::IntentDrift,
        },
    )
}
