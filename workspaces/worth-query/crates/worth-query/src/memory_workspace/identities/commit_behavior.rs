use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::sync::Arc;
use worth_foundational::facade::admit_foundational_authority_identity;
use worth_runtime_bridge::facade::{BridgeIdentityEvidence, TruthCommitIdentity};

use super::{WorthQueryCommitIdentity, WorthQueryCommitIdentityInner};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::identity_authority::query_receipt_admission_authority;

impl WorthQueryCommitIdentity {
    pub(crate) fn bridge_identity(&self) -> Option<&TruthCommitIdentity> {
        match &self.inner {
            WorthQueryCommitIdentityInner::Absent => None,
            WorthQueryCommitIdentityInner::RelationalBridge { bridge_identity } => {
                Some(bridge_identity)
            }
            WorthQueryCommitIdentityInner::Preview { .. } => None,
        }
    }

    pub fn evidence_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(match &self.inner {
            WorthQueryCommitIdentityInner::Absent => WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::WriteReceiptCommitIdentity,
            )
            .field_shape(WorthQueryEvidenceTag::new("commit_state"), "absent")
            .seal()?,
            WorthQueryCommitIdentityInner::RelationalBridge { bridge_identity } => {
                let commit_id = bridge_identity
                    .relational_commit_id()
                    .expect("worth-query commit identity must retain relational commit payload");
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::WriteReceiptCommitIdentity,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("relational_commit_id"),
                    commit_id as usize,
                )
                .seal()?
            }
            WorthQueryCommitIdentityInner::Preview { evidence_identity } => {
                evidence_identity.clone()
            }
        })
    }

    pub fn bridge_admission_evidence(
        &self,
    ) -> Result<BridgeIdentityEvidence, IdentityDerivationDenial> {
        Ok(match &self.inner {
            WorthQueryCommitIdentityInner::RelationalBridge { bridge_identity } => {
                bridge_identity.bridge_admission_evidence()
            }
            _ => self.evidence_identity()?.bridge_evidence_identity(),
        })
    }

    pub fn terminal_projection_for_reporting(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self
            .evidence_identity()?
            .terminal_projection_for_reporting()
            .to_string())
    }

    pub fn is_same_current_identity_as(&self, candidate: &Self) -> bool {
        self._authority.is_some() && candidate._authority.is_some() && self.inner == candidate.inner
    }

    pub(crate) fn has_current_authority(&self) -> bool {
        self._authority.is_some()
    }

    pub(crate) fn admit_runtime_write_authority(
        mut self,
    ) -> Result<Self, IdentityDerivationDenial> {
        if self._authority.is_none() {
            let basis = self.evidence_identity()?;
            self._authority = Some(admit_foundational_authority_identity(
                Arc::<str>::from(basis.terminal_projection_for_reporting()),
                query_receipt_admission_authority(),
            ));
        }
        Ok(self)
    }

    pub(crate) fn relational_commit_id(&self) -> Option<u64> {
        self.bridge_identity()
            .and_then(TruthCommitIdentity::relational_commit_id)
    }

    pub(crate) fn preview_evidence_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        match &self.inner {
            WorthQueryCommitIdentityInner::Preview { evidence_identity } => Some(evidence_identity),
            _ => None,
        }
    }
}

impl std::fmt::Debug for WorthQueryCommitIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let posture = if self._authority.is_some() {
            "current"
        } else {
            "projection"
        };
        formatter
            .debug_struct("WorthQueryCommitIdentity")
            .field("posture", &posture)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryCommitIdentity {
    fn eq(&self, candidate: &Self) -> bool {
        self.inner == candidate.inner
    }
}

impl Eq for WorthQueryCommitIdentity {}
