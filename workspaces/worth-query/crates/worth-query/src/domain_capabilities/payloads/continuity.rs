use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use crate::runtime::{
    WorthQueryContinuityMutationFamily, WorthQueryContinuityMutationOutcomeClass,
};

use super::common::{
    SealedPayload, WorthQueryDomainCapabilityCategory, WorthQueryDomainCapabilityPayload,
    WorthQueryDomainCapabilitySemanticPosture,
};
use super::continuity_correspondence::WorthQueryContinuityCorrespondenceSemantics;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryContinuityContributionPosture {
    Preserved,
    Split,
    Replaced,
    CorrespondenceOnly,
}

impl WorthQueryContinuityContributionPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preserved => "preserved",
            Self::Split => "split",
            Self::Replaced => "replaced",
            Self::CorrespondenceOnly => "correspondence-only",
        }
    }

    pub const fn semantic_posture(self) -> WorthQueryDomainCapabilitySemanticPosture {
        match self {
            Self::Preserved => WorthQueryDomainCapabilitySemanticPosture::ContinuityPreserved,
            Self::Split => WorthQueryDomainCapabilitySemanticPosture::ContinuitySplit,
            Self::Replaced => WorthQueryDomainCapabilitySemanticPosture::ContinuityReplaced,
            Self::CorrespondenceOnly => {
                WorthQueryDomainCapabilitySemanticPosture::ContinuityCorrespondenceOnly
            }
        }
    }
}

fn continuity_authoritative_identity(
    role: &str,
    source_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_continuity_authoritative_source_v1")
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label)
            .seal()?,
    )
}

fn continuity_successor_authoritative_identity(
    index: usize,
    source_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_continuity_authoritative_source_v1")
            .field_shape(WorthQueryEvidenceTag::new("role"), "successor")
            .field_usize(WorthQueryEvidenceTag::new("index"), index)
            .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label)
            .seal()?,
    )
}

fn compose_continuity_runtime_semantics_identity(
    runtime_semantics: &WorthQueryContinuityRuntimeSemantics,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(domain_capability_scope_encoder(
        "worth_query_domain_capability_continuity_runtime_semantics_v1",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("family"),
        runtime_semantics.family().as_str(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("outcome_class"),
        runtime_semantics.outcome_class().as_str(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("prior_authoritative"),
        runtime_semantics.prior_authoritative_identity(),
    )
    .field_evidence_identity_sequence(
        WorthQueryEvidenceTag::new("successor_authoritative"),
        runtime_semantics
            .successor_authoritative_identities()
            .iter(),
    )
    .seal()?)
}

fn compose_continuity_payload_identity(
    posture: WorthQueryContinuityContributionPosture,
    semantic_code: &str,
    detail: &str,
    runtime_semantics: Option<&WorthQueryContinuityRuntimeSemantics>,
    correspondence_semantics: Option<&WorthQueryContinuityCorrespondenceSemantics>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity = domain_capability_scope_encoder("worth_query_domain_capability_payload_v3")
        .field_shape(
            WorthQueryEvidenceTag::new("category"),
            WorthQueryDomainCapabilityCategory::ContinuityLineage.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
        .field_shape(WorthQueryEvidenceTag::new("semantic_code"), semantic_code)
        .field_shape(WorthQueryEvidenceTag::new("detail"), detail);
    identity = match runtime_semantics {
        Some(runtime) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("runtime"),
            &compose_continuity_runtime_semantics_identity(runtime)?,
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("runtime"), "none"),
    };
    identity = match correspondence_semantics {
        Some(correspondence) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("correspondence"),
            &correspondence.semantics_identity(),
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("correspondence"), "none"),
    };
    Ok(identity.seal()?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContinuityRuntimeSemantics {
    family: WorthQueryContinuityMutationFamily,
    outcome_class: WorthQueryContinuityMutationOutcomeClass,
    prior_authoritative_source_label_for_reporting: String,
    prior_authoritative_identity: WorthQueryEvidenceIdentity,
    successor_authoritative_source_labels_for_reporting: Vec<String>,
    successor_authoritative_identities: Vec<WorthQueryEvidenceIdentity>,
}

impl WorthQueryContinuityRuntimeSemantics {
    pub fn new<I, S>(
        family: WorthQueryContinuityMutationFamily,
        outcome_class: WorthQueryContinuityMutationOutcomeClass,
        prior_authoritative_identity: impl Into<String>,
        successor_authoritative_identities: I,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let prior_authoritative_source_label_for_reporting = prior_authoritative_identity.into();
        let prior_label = prior_authoritative_source_label_for_reporting.clone();
        let successor_labels = successor_authoritative_identities
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        let successor_authoritative_identities = successor_labels
            .iter()
            .enumerate()
            .map(|(index, label)| continuity_successor_authoritative_identity(index, label))
            .collect();
        Self {
            family,
            outcome_class,
            prior_authoritative_source_label_for_reporting,
            prior_authoritative_identity: continuity_authoritative_identity("prior", &prior_label),
            successor_authoritative_source_labels_for_reporting: successor_labels,
            successor_authoritative_identities,
        }
    }

    pub fn family(&self) -> WorthQueryContinuityMutationFamily {
        self.family
    }

    pub fn outcome_class(&self) -> WorthQueryContinuityMutationOutcomeClass {
        self.outcome_class
    }

    pub fn prior_authoritative_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.prior_authoritative_identity
    }

    pub fn prior_authoritative_for_reporting(&self) -> &str {
        self.prior_authoritative_identity.as_str()
    }

    pub fn prior_authoritative_source_label_for_reporting(&self) -> &str {
        &self.prior_authoritative_source_label_for_reporting
    }

    pub fn successor_authoritative_identities(&self) -> &[WorthQueryEvidenceIdentity] {
        &self.successor_authoritative_identities
    }

    pub fn successor_authoritative_source_labels_for_reporting(&self) -> &[String] {
        &self.successor_authoritative_source_labels_for_reporting
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContinuityContributionPayload {
    posture: WorthQueryContinuityContributionPosture,
    semantic_code: String,
    detail: String,
    runtime_semantics: Option<WorthQueryContinuityRuntimeSemantics>,
    correspondence_semantics: Option<WorthQueryContinuityCorrespondenceSemantics>,
    payload_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryContinuityContributionPayload {
    pub fn new(
        posture: WorthQueryContinuityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            posture,
            semantic_code,
            detail,
            None,
        )?)
    }

    pub fn with_runtime_semantics(
        posture: WorthQueryContinuityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryContinuityRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_all_semantics(
            posture,
            semantic_code,
            detail,
            runtime_semantics,
            None,
        )?)
    }

    pub fn with_correspondence_semantics(
        posture: WorthQueryContinuityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        correspondence_semantics: Option<WorthQueryContinuityCorrespondenceSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_all_semantics(
            posture,
            semantic_code,
            detail,
            None,
            correspondence_semantics,
        )?)
    }

    pub fn with_all_semantics(
        posture: WorthQueryContinuityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryContinuityRuntimeSemantics>,
        correspondence_semantics: Option<WorthQueryContinuityCorrespondenceSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let semantic_code = semantic_code.into();
        let detail = detail.into();
        let payload_identity = compose_continuity_payload_identity(
            posture,
            &semantic_code,
            &detail,
            runtime_semantics.as_ref(),
            correspondence_semantics.as_ref(),
        )?;
        Ok(Self {
            posture,
            semantic_code,
            detail,
            runtime_semantics,
            correspondence_semantics,
            payload_identity,
        })
    }

    pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
        WorthQueryDomainCapabilityCategory::ContinuityLineage
    }

    pub fn posture(&self) -> WorthQueryContinuityContributionPosture {
        self.posture
    }

    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn runtime_semantics(&self) -> Option<&WorthQueryContinuityRuntimeSemantics> {
        self.runtime_semantics.as_ref()
    }

    pub fn correspondence_semantics(&self) -> Option<&WorthQueryContinuityCorrespondenceSemantics> {
        self.correspondence_semantics.as_ref()
    }

    pub fn payload_digest(&self) -> &str {
        self.payload_identity.as_str()
    }

    pub fn payload_for_reporting(&self) -> &str {
        self.payload_identity.as_str()
    }

    pub fn payload_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.payload_identity
    }
}

impl SealedPayload for WorthQueryContinuityContributionPayload {}

impl WorthQueryDomainCapabilityPayload for WorthQueryContinuityContributionPayload {
    fn category(&self) -> WorthQueryDomainCapabilityCategory {
        self.category()
    }

    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }

    fn semantic_posture(&self) -> WorthQueryDomainCapabilitySemanticPosture {
        self.posture().semantic_posture()
    }

    fn semantic_code(&self) -> &str {
        self.semantic_code()
    }

    fn detail(&self) -> &str {
        self.detail()
    }

    fn payload_digest(&self) -> &str {
        self.payload_digest()
    }

    fn payload_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.payload_identity
    }
}
