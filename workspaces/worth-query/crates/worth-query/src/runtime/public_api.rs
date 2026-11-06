use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryAuthorityLane, WorthQueryRuntimeBackendPosture, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupportStatus, WorthQueryRuntimeFamilyTeachingPosture,
    WorthQueryRuntimeSupportProfile,
};
#[path = "public_api_naming.rs"]
mod naming;
#[path = "public_api_transcript.rs"]
mod transcript;

pub use naming::{WorthQueryRuntimePublicApiNamingContract, WorthQueryRuntimePublicApiNamingRow};
pub use transcript::WorthQueryRuntimePublicApiTranscriptEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimePublicApiFamilyContract {
    family: WorthQueryRuntimeFacadeFamily,
    status: WorthQueryRuntimeFamilySupportStatus,
    teaching_posture: WorthQueryRuntimeFamilyTeachingPosture,
    authority_lanes: Vec<WorthQueryAuthorityLane>,
    evidence: Vec<String>,
    reason: Option<String>,
    owner_closure: String,
    extension_rule: String,
    parallel_api_forbidden: bool,
    admission_fail_closed: bool,
    contract_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimePublicApiFamilyContract {
    fn from_support_row(
        row: &super::WorthQueryRuntimeFamilySupport,
    ) -> Result<Self, IdentityDerivationDenial> {
        let family = row.family();
        let status = row.status();
        let teaching_posture = row.teaching_posture();
        let authority_lanes = row.authority_lanes().to_vec();
        let evidence = row.evidence().to_vec();
        let reason = row.denial_reason().map(str::to_string);
        let owner_closure = row.owner_closure().to_string();
        let extension_rule = row.extension_rule().to_string();
        let parallel_api_forbidden = row.parallel_api_forbidden();
        let admission_fail_closed = row.admission_fail_closed();
        let contract_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicApiFamilyContract)
                .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
                .field_shape(WorthQueryEvidenceTag::new("status"), status.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("teaching_posture"),
                    teaching_posture.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("owner_closure"),
                    owner_closure.clone(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("extension_rule"),
                    extension_rule.clone(),
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("parallel_api_forbidden"),
                    parallel_api_forbidden,
                )
                .field_bool(
                    WorthQueryEvidenceTag::new("admission_fail_closed"),
                    admission_fail_closed,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("authority_lanes"),
                    authority_lanes.iter().map(|lane| lane.as_str()),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("evidence"),
                    evidence.iter().map(String::as_str),
                )
                .optional_value(WorthQueryEvidenceTag::new("reason"), reason.as_deref())
                .seal()?;
        Ok(Self {
            family,
            status,
            teaching_posture,
            authority_lanes,
            evidence,
            reason,
            owner_closure,
            extension_rule,
            parallel_api_forbidden,
            admission_fail_closed,
            contract_identity,
        })
    }

    pub fn family(&self) -> WorthQueryRuntimeFacadeFamily {
        self.family
    }

    pub fn status(&self) -> WorthQueryRuntimeFamilySupportStatus {
        self.status
    }

    pub fn teaching_posture(&self) -> WorthQueryRuntimeFamilyTeachingPosture {
        self.teaching_posture
    }

    pub fn ordinary_downstream_dx(&self) -> bool {
        self.teaching_posture == WorthQueryRuntimeFamilyTeachingPosture::OrdinaryRuntimeDx
    }

    pub fn authority_lanes(&self) -> &[WorthQueryAuthorityLane] {
        &self.authority_lanes
    }

    pub fn evidence(&self) -> &[String] {
        &self.evidence
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn owner_closure(&self) -> &str {
        &self.owner_closure
    }

    pub fn extension_rule(&self) -> &str {
        &self.extension_rule
    }

    pub fn parallel_api_forbidden(&self) -> bool {
        self.parallel_api_forbidden
    }

    pub fn admission_fail_closed(&self) -> bool {
        self.admission_fail_closed
    }

    pub fn contract_digest(&self) -> &str {
        self.contract_identity.as_str()
    }

    pub fn contract_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.contract_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimePublicApiContract {
    backend_posture: WorthQueryRuntimeBackendPosture,
    families: Vec<WorthQueryRuntimePublicApiFamilyContract>,
    stable_family_count: usize,
    deferred_family_count: usize,
    unsupported_family_count: usize,
    contract_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimePublicApiContract {
    pub fn from_support_profile(
        profile: &WorthQueryRuntimeSupportProfile,
    ) -> Result<Self, IdentityDerivationDenial> {
        let families: Vec<_> = profile
            .rows()
            .map(WorthQueryRuntimePublicApiFamilyContract::from_support_row)
            .collect()?;
        let stable_family_count = families
            .iter()
            .filter(|family| family.status() == WorthQueryRuntimeFamilySupportStatus::Supported)
            .count();
        let deferred_family_count = families
            .iter()
            .filter(|family| family.status() == WorthQueryRuntimeFamilySupportStatus::DeferredDebt)
            .count();
        let unsupported_family_count = families
            .iter()
            .filter(|family| family.status() == WorthQueryRuntimeFamilySupportStatus::Unsupported)
            .count();
        let contract_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimePublicApiContract)
                .field_shape(
                    WorthQueryEvidenceTag::new("backend_posture"),
                    profile.posture().as_str(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("family_contract_digest"),
                    families
                        .iter()
                        .map(WorthQueryRuntimePublicApiFamilyContract::contract_digest),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("stable_family_count"),
                    stable_family_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("deferred_family_count"),
                    deferred_family_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("unsupported_family_count"),
                    unsupported_family_count,
                )
                .seal()?;
        Ok(Self {
            backend_posture: profile.posture(),
            families,
            stable_family_count,
            deferred_family_count,
            unsupported_family_count,
            contract_identity,
        })
    }

    pub fn backend_posture(&self) -> WorthQueryRuntimeBackendPosture {
        self.backend_posture
    }

    pub fn families(&self) -> &[WorthQueryRuntimePublicApiFamilyContract] {
        &self.families
    }

    pub fn stable_family_count(&self) -> usize {
        self.stable_family_count
    }

    pub fn deferred_family_count(&self) -> usize {
        self.deferred_family_count
    }

    pub fn unsupported_family_count(&self) -> usize {
        self.unsupported_family_count
    }

    pub fn contract_digest(&self) -> &str {
        self.contract_identity.as_str()
    }

    pub fn contract_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.contract_identity
    }

    pub fn family(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Option<&WorthQueryRuntimePublicApiFamilyContract> {
        self.families.iter().find(|row| row.family() == family)
    }
}
