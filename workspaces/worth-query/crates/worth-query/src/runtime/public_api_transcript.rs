use crate::evidence_identity::WorthQueryEvidenceIdentity;
#[cfg(test)]
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimePublicApiTranscriptEvidence {
    transcript_family: String,
    support_contract_digest: String,
    state_digest: String,
    live_surface_digest: String,
    computed_surface_digest: String,
    effect_surface_digest: String,
    intent_receipt_digest: String,
    inspection_digest: String,
    support_gated_neighbor_denial_digests: Vec<String>,
    delivery_residue_count: usize,
    authority_lane_digest: String,
    meaningful_assertion_count: usize,
    transcript_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimePublicApiTranscriptEvidence {
    #[cfg(test)]
    pub(crate) fn new(
        transcript_family: impl Into<String>,
        support_contract_digest: impl AsRef<str>,
        state_digest: impl AsRef<str>,
        live_surface_digest: impl AsRef<str>,
        computed_surface_digest: impl AsRef<str>,
        effect_surface_digest: impl AsRef<str>,
        intent_receipt_digest: impl AsRef<str>,
        inspection_digest: impl AsRef<str>,
        support_gated_neighbor_denial_digests: impl IntoIterator<Item = impl AsRef<str>>,
        delivery_residue_count: usize,
        authority_lane_digest: impl AsRef<str>,
        meaningful_assertion_count: usize,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let transcript_family = transcript_family.into();
        let support_contract_digest = support_contract_digest.as_ref().to_string();
        let state_digest = state_digest.as_ref().to_string();
        let live_surface_digest = live_surface_digest.as_ref().to_string();
        let computed_surface_digest = computed_surface_digest.as_ref().to_string();
        let effect_surface_digest = effect_surface_digest.as_ref().to_string();
        let intent_receipt_digest = intent_receipt_digest.as_ref().to_string();
        let inspection_digest = inspection_digest.as_ref().to_string();
        let support_gated_neighbor_denial_digests = support_gated_neighbor_denial_digests
            .into_iter()
            .map(|digest| digest.as_ref().to_string())
            .collect::<Vec<_>>();
        let authority_lane_digest = authority_lane_digest.as_ref().to_string();
        assert!(
            !support_gated_neighbor_denial_digests.is_empty(),
            "runtime public API transcript evidence must prove at least one support-gated neighbor denial"
        );
        let transcript_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::RuntimePublicApiTranscriptEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("transcript_family"),
            transcript_family.clone(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("support_contract_digest"),
            support_contract_digest.clone(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("state_digest"),
            state_digest.clone(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("live_surface_digest"),
            live_surface_digest.clone(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("computed_surface_digest"),
            computed_surface_digest.clone(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("effect_surface_digest"),
            effect_surface_digest.clone(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("intent_receipt_digest"),
            intent_receipt_digest.clone(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("inspection_digest"),
            inspection_digest.clone(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("support_gated_neighbor_denial_digest"),
            support_gated_neighbor_denial_digests
                .iter()
                .map(String::as_str),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("delivery_residue_count"),
            delivery_residue_count,
        )
        .field_value(
            WorthQueryEvidenceTag::new("authority_lane_digest"),
            authority_lane_digest.clone(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("meaningful_assertion_count"),
            meaningful_assertion_count,
        )
        .seal()?;
        Ok(Self {
            transcript_family,
            support_contract_digest,
            state_digest,
            live_surface_digest,
            computed_surface_digest,
            effect_surface_digest,
            intent_receipt_digest,
            inspection_digest,
            support_gated_neighbor_denial_digests,
            delivery_residue_count,
            authority_lane_digest,
            meaningful_assertion_count,
            transcript_identity,
        })
    }

    pub fn transcript_family(&self) -> &str {
        &self.transcript_family
    }

    pub fn support_contract_digest(&self) -> &str {
        &self.support_contract_digest
    }

    pub fn state_digest(&self) -> &str {
        &self.state_digest
    }

    pub fn live_surface_digest(&self) -> &str {
        &self.live_surface_digest
    }

    pub fn computed_surface_digest(&self) -> &str {
        &self.computed_surface_digest
    }

    pub fn effect_surface_digest(&self) -> &str {
        &self.effect_surface_digest
    }

    pub fn intent_receipt_digest(&self) -> &str {
        &self.intent_receipt_digest
    }

    pub fn inspection_digest(&self) -> &str {
        &self.inspection_digest
    }

    pub fn support_gated_neighbor_denial_digests(&self) -> &[String] {
        &self.support_gated_neighbor_denial_digests
    }

    pub fn delivery_residue_count(&self) -> usize {
        self.delivery_residue_count
    }

    pub fn authority_lane_digest(&self) -> &str {
        &self.authority_lane_digest
    }

    pub fn meaningful_assertion_count(&self) -> usize {
        self.meaningful_assertion_count
    }

    pub fn transcript_digest(&self) -> &str {
        self.transcript_identity.as_str()
    }

    pub fn transcript_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.transcript_identity
    }
}
