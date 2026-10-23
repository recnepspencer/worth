//! Private replay lookup discloses the actual performed inverse input without resuming effects.
use super::intent::WorthQueryCorrectionIntent;
use super::*;
use crate::domain_computation::primary_graph::provider::CorrectionIdempotencyObservation as Observed;
use worth_query_declaration::facade::{
    application_aftermath::ApplicationInverseBinding,
    application_operation::{ApplicationEncodedInput, ApplicationMutationBinding},
};

/// Correction-command lookup under fresh disclosure for the chosen action binding.
///
/// A first use must still derive and admit its action. A replay carries the exact
/// originally encoded action input and recorded intent for fresh admission; it
/// neither substitutes current lineage edges nor resumes publication or recovery.
pub enum WorthQueryCorrectionActionInputSelection<
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    /// No prior command was found; first-use eligibility and ordinary admission must follow.
    FirstUse(WorthQueryCorrectionIntent),
    /// The completed command's immutable intent and exact performed input.
    Replay {
        intent: WorthQueryCorrectionIntent,
        input: ApplicationEncodedInput<Binding::InputBinding>,
    },
    /// This command key already names different immutable correction meaning.
    IntentDrift,
    /// The existing publication is still owned by its original pending operation.
    PublicationPending,
    /// Existing owner effects await their original product-publication recovery.
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}
/// Input selection for a reversal's declared inverse binding.
pub type WorthQueryCorrectionInputSelection<S, C> =
    WorthQueryCorrectionActionInputSelection<S, ApplicationInverseBinding<S, C>>;

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
    let observed = target
        .application()
        .primary_provider
        .observe_correction_key(target.product().observation(), &key.identity());
    let lineage = match &observed {
        Observed::Completed(_, committed) => committed
            .aftermath_causality()
            .map(|fact| fact.correction_lineage())
            .cloned()
            .ok_or(WorthQueryCorrectionDerivationDenial::UnsupportedSourceMeaning)?,
        _ => super::lineage::reverse_identity(source)?,
    };
    if lineage.role() != crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole::RecordedReverse {
        return Ok(WorthQueryCorrectionActionInputSelection::IntentDrift);
    }
    let intent = WorthQueryCorrectionIntent::from_disclosed_source::<Schema, C>(
        target, source, key, &lineage,
    );
    select_action::<Schema, ApplicationInverseBinding<Schema, C>>(observed, intent, key)
}

pub(super) fn select_reapplication_input<
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    alternative: &super::WorthQuerySelectedCorrectionAlternative<'_, Schema>,
    key: &WorthQueryCorrectionCommandKey,
    maximum_history: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
) -> Result<
    WorthQueryCorrectionActionInputSelection<Schema, C::Source>,
    WorthQueryCorrectionDerivationDenial,
> {
    use WorthQueryCorrectionDerivationDenial as Denial;
    disclosure::disclose_correction_source::<Schema, C>(
        target,
        phase,
        principal,
        request,
        &alternative.source,
        maximum_history,
        maximum_results,
    )?;
    target.select_alternative_ancestry(phase, &alternative.selected, maximum_history)?;
    let observed = target
        .application()
        .primary_provider
        .observe_correction_key(target.product().observation(), &key.identity());
    let lineage = match &observed {
        Observed::Completed(_, committed) => committed
            .aftermath_causality()
            .map(|fact| fact.correction_lineage())
            .cloned()
            .ok_or(Denial::UnsupportedSourceMeaning)?,
        _ => super::reapplication::selected_lineage(alternative)?,
    };
    if lineage.role() != crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole::RecordedReapply
        || lineage.root() != &alternative.root || lineage.selected_alternative() != Some(&alternative.selected)
    { return Ok(WorthQueryCorrectionActionInputSelection::IntentDrift); }
    target.select_alternative_ancestry(
        phase,
        lineage
            .active_reversal()
            .ok_or(Denial::UnsupportedSourceMeaning)?,
        maximum_history,
    )?;
    let intent = WorthQueryCorrectionIntent::from_disclosed_source::<Schema, C>(
        target,
        &alternative.source,
        key,
        &lineage,
    );
    select_action::<Schema, C::Source>(observed, intent, key)
}

fn select_action<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    observed: Observed,
    intent: WorthQueryCorrectionIntent,
    key: &WorthQueryCorrectionCommandKey,
) -> Result<
    WorthQueryCorrectionActionInputSelection<Schema, Binding>,
    WorthQueryCorrectionDerivationDenial,
> {
    use WorthQueryCorrectionActionInputSelection as Selection;
    Ok(match observed {
        Observed::Unseen => Selection::FirstUse(intent),
        Observed::Completed(binding, committed) => {
            if !intent.matches(&binding) || !binding.is_for_mutation_binding(Binding::IDENTITY) {
                return Ok(Selection::IntentDrift);
            }
            let input = committed
                .commit_evidence()
                .original_authority()
                .retained_governed_input_carrier()
                .and_then(|retained| retained.encoded_input::<Binding::InputBinding>())
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
    })
}
