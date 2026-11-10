use crate::application::{
    WorthQueryDeclarationEntryContributionCategoryFamily,
    WorthQueryDeclarationEntryContributionComposition, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::super::artifact::{
    WorthQueryContributionComposedContribution, WorthQueryContributionComposedOrchestration,
};
use super::super::composition::{
    classify_intent_results, strongest_stop, WorthQueryContributionComposedComposition,
    WorthQueryContributionComposedStop,
};
use super::super::input::{
    WorthQueryContributionComposedMaterializationPolicy,
    WorthQueryContributionComposedOrchestrationInput,
};
use super::super::intent_result::WorthQueryContributionComposedIntentResult;

pub(crate) fn request_descriptor<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    input: &WorthQueryContributionComposedOrchestrationInput<D, I>,
) -> String {
    format!(
        "{}:{}:{}:{:?}",
        I::Family::semantic_family_key(),
        input
            .declaration_input()
            .canonical_declaration_entries()
            .len(),
        input.contributions().len(),
        input.materialization_policy()
    )
}

pub(crate) fn request_identity<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>(
    input: &WorthQueryContributionComposedOrchestrationInput<D, I>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let contribution_identities = input
        .contributions()
        .iter()
        .enumerate()
        .map(|(index, value)| {
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_contribution_composed_request_intent_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("order"), index.to_string())
                .field_shape(WorthQueryEvidenceTag::new("intent"), format!("{value:?}"))
                .seal()
        })
        .collect::<Result<Vec<_>, _>>()?;
    worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_contribution_composed_request_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            I::Family::semantic_family_key(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("declaration_entries"),
            format!(
                "{:?}",
                input.declaration_input().canonical_declaration_entries()
            ),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("contribution_count"),
            input.contributions().len().to_string(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("contributions"),
            contribution_identities.iter(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("materialization"),
            format!("{:?}", input.materialization_policy()),
        )
        .seal()
}

pub(crate) fn build_composed_artifact<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
>(
    envelope: crate::application::WorthQueryDeclarationEnvelope<D, I>,
    intent_results: Vec<WorthQueryContributionComposedIntentResult>,
) -> Result<
    Result<
        WorthQueryContributionComposedOrchestration<D, I>,
        (WorthQueryContributionComposedStop, String),
    >,
    IdentityDerivationDenial,
> {
    let classification = classify_intent_results(&intent_results);
    let composition = WorthQueryContributionComposedComposition::from_intent_results(
        classification,
        &intent_results,
    )?;
    let contribution_composition = WorthQueryDeclarationEntryContributionComposition::new(
        composition.admitted_evidence().to_vec(),
        composition.rejected_category_families().to_vec(),
    );
    let contributions = intent_results
        .iter()
        .filter_map(|value| value.contribution().cloned())
        .collect::<Vec<WorthQueryContributionComposedContribution>>();
    if contributions.is_empty() && !intent_results.is_empty() {
        return Ok(Err((
            strongest_stop(&intent_results),
            composition.composition_for_reporting().to_string(),
        )));
    }
    Ok(Ok(WorthQueryContributionComposedOrchestration::new(
        envelope,
        contribution_composition,
        contributions,
        intent_results,
        composition,
    )))
}

pub(crate) fn stop_reason(
    stop: WorthQueryContributionComposedStop,
    intent_results: &[WorthQueryContributionComposedIntentResult],
) -> String {
    let rejected_families = intent_results
        .iter()
        .filter(|value| !value.is_admitted())
        .map(WorthQueryContributionComposedIntentResult::category_family)
        .map(WorthQueryDeclarationEntryContributionCategoryFamily::as_str)
        .collect::<Vec<_>>();
    let stop_label = match stop {
        WorthQueryContributionComposedStop::Deferred => "deferred",
        WorthQueryContributionComposedStop::DeclarationDenied => "declaration-denied",
        WorthQueryContributionComposedStop::ContributionDenied => "contribution-denied",
        WorthQueryContributionComposedStop::Stale => "stale",
        WorthQueryContributionComposedStop::RebindRequired => "rebind-required",
        WorthQueryContributionComposedStop::Unsupported => "unsupported",
        WorthQueryContributionComposedStop::Failed => "failed",
    };
    format!(
        "contribution composition stopped as {stop_label}; rejected families: {}",
        if rejected_families.is_empty() {
            "none".to_string()
        } else {
            rejected_families.join(", ")
        }
    )
}

pub(crate) fn materialization_policy_label(
    policy: &WorthQueryContributionComposedMaterializationPolicy,
) -> &'static str {
    match policy {
        WorthQueryContributionComposedMaterializationPolicy::None => "none",
        WorthQueryContributionComposedMaterializationPolicy::Summary(_) => "summary",
    }
}
