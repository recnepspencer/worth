use crate::application::{
    WorthQueryDeclarationEntryContributionCategoryFamily,
    WorthQueryDeclarationEntryContributionEvidence,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::intent_result::{
    WorthQueryContributionComposedIntentClassification, WorthQueryContributionComposedIntentResult,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryContributionComposedClassification {
    FullyAdmitted,
    PartiallyAdmitted,
    NoContributionAdmitted,
    MaterializationFailedAfterAdmission,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryContributionComposedStop {
    Deferred,
    DeclarationDenied,
    ContributionDenied,
    Stale,
    RebindRequired,
    Unsupported,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContributionComposedComposition {
    classification: WorthQueryContributionComposedClassification,
    admitted_category_families: Vec<WorthQueryDeclarationEntryContributionCategoryFamily>,
    rejected_category_families: Vec<WorthQueryDeclarationEntryContributionCategoryFamily>,
    admitted_evidence: Vec<WorthQueryDeclarationEntryContributionEvidence>,
    composition_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryContributionComposedComposition {
    pub fn from_intent_results(
        classification: WorthQueryContributionComposedClassification,
        intent_results: &[WorthQueryContributionComposedIntentResult],
    ) -> Result<Self, IdentityDerivationDenial> {
        let admitted_evidence = intent_results
            .iter()
            .filter_map(|value| value.contribution().map(|entry| entry.evidence().clone()))
            .collect::<Vec<_>>();
        let admitted_category_families = admitted_evidence
            .iter()
            .map(WorthQueryDeclarationEntryContributionEvidence::category_family)
            .collect::<Vec<_>>();
        let rejected_category_families = intent_results
            .iter()
            .filter(|value| !value.is_admitted())
            .map(WorthQueryContributionComposedIntentResult::category_family)
            .collect::<Vec<_>>();
        let composition_identity = compose_composition_identity(classification, intent_results)?;
        Ok(Self {
            classification,
            admitted_category_families,
            rejected_category_families,
            admitted_evidence,
            composition_identity,
        })
    }

    pub fn classification(&self) -> WorthQueryContributionComposedClassification {
        self.classification
    }

    pub fn admitted_category_families(
        &self,
    ) -> &[WorthQueryDeclarationEntryContributionCategoryFamily] {
        &self.admitted_category_families
    }

    pub fn rejected_category_families(
        &self,
    ) -> &[WorthQueryDeclarationEntryContributionCategoryFamily] {
        &self.rejected_category_families
    }

    pub fn admitted_evidence(&self) -> &[WorthQueryDeclarationEntryContributionEvidence] {
        &self.admitted_evidence
    }

    pub fn composition_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.composition_identity
    }

    pub fn composition_for_reporting(&self) -> &str {
        self.composition_identity.as_str()
    }
}

fn compose_intent_result_identity(
    value: &WorthQueryContributionComposedIntentResult,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_contribution_composed_intent_result_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("category_family"),
                value.category_family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("classification"),
                format!("{:?}", value.classification()),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("request"),
                value.request_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("binding"),
                value.binding_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("declaration_contract"),
                format!("{:?}", value.aspect_record().declaration_contract()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("declaration_coverage"),
                format!("{:?}", value.aspect_record().declaration_coverage()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("evaluation"),
                value.evaluation().stage_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("admission"),
                value.admission().stage_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("materialization"),
                value
                    .materialization()
                    .stage_identity()
                    .or_else(|| value.admission().stage_identity())
                    .or_else(|| value.evaluation().stage_identity()),
            )
            .seal()?,
    )
}

fn compose_composition_identity(
    classification: WorthQueryContributionComposedClassification,
    intent_results: &[WorthQueryContributionComposedIntentResult],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut intent_identities = intent_results
        .iter()
        .map(compose_intent_result_identity)
        .collect::<Result<Vec<_>, _>>()?;
    intent_identities.sort_by_key(|identity| identity.as_str().to_owned());
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_contribution_composed_composition_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("classification"),
                format!("{classification:?}"),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("intent_results"),
                intent_identities.iter(),
            )
            .seal()?,
    )
}

pub fn classify_intent_results(
    intent_results: &[WorthQueryContributionComposedIntentResult],
) -> WorthQueryContributionComposedClassification {
    let admitted_count = intent_results
        .iter()
        .filter(|value| value.is_admitted())
        .count();
    let has_materialization_failure = intent_results.iter().any(|value| {
        value.classification()
            == WorthQueryContributionComposedIntentClassification::MaterializationFailedAfterAdmission
    });
    if admitted_count == intent_results.len() {
        WorthQueryContributionComposedClassification::FullyAdmitted
    } else if has_materialization_failure {
        WorthQueryContributionComposedClassification::MaterializationFailedAfterAdmission
    } else if admitted_count == 0 {
        WorthQueryContributionComposedClassification::NoContributionAdmitted
    } else {
        WorthQueryContributionComposedClassification::PartiallyAdmitted
    }
}

pub fn strongest_stop(
    intent_results: &[WorthQueryContributionComposedIntentResult],
) -> WorthQueryContributionComposedStop {
    if intent_results.iter().any(|value| {
        value.classification() == WorthQueryContributionComposedIntentClassification::Denied
    }) {
        WorthQueryContributionComposedStop::ContributionDenied
    } else if intent_results.iter().any(|value| {
        value.classification() == WorthQueryContributionComposedIntentClassification::Stale
    }) {
        WorthQueryContributionComposedStop::Stale
    } else if intent_results.iter().any(|value| {
        value.classification() == WorthQueryContributionComposedIntentClassification::RebindRequired
    }) {
        WorthQueryContributionComposedStop::RebindRequired
    } else if intent_results.iter().any(|value| {
        value.classification() == WorthQueryContributionComposedIntentClassification::Unsupported
    }) {
        WorthQueryContributionComposedStop::Unsupported
    } else {
        WorthQueryContributionComposedStop::Failed
    }
}
