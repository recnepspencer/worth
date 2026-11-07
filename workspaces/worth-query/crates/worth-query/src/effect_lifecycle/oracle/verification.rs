use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::{BridgeExecutionOracle, RelationalExecutionOracle};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectExecutionOracleVerificationKind {
    Mutation,
    Merge,
    Writeback,
    MutationBatch,
}

impl EffectExecutionOracleVerificationKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Mutation => "mutation",
            Self::Merge => "merge",
            Self::Writeback => "writeback",
            Self::MutationBatch => "mutation_batch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectExecutionOracleVerification {
    verification_kind: EffectExecutionOracleVerificationKind,
    execution_subject_identity: WorthQueryEvidenceIdentity,
    relational_oracle_identity: Option<WorthQueryEvidenceIdentity>,
    bridge_oracle_identity: Option<WorthQueryEvidenceIdentity>,
    verification_identity: WorthQueryEvidenceIdentity,
    component_count: usize,
}

impl EffectExecutionOracleVerification {
    pub(crate) fn relational(
        verification_kind: EffectExecutionOracleVerificationKind,
        execution_subject_identity: &WorthQueryEvidenceIdentity,
        oracle: &RelationalExecutionOracle,
        component_count: usize,
    ) -> Self {
        let verification_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_oracle_verification_v1",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("kind"),
                    verification_kind.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("execution_subject"),
                    execution_subject_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("relational_oracle"),
                    oracle.relational_oracle_identity(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("component_count"),
                    component_count,
                )
                .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            verification_kind,
            execution_subject_identity: execution_subject_identity.clone(),
            relational_oracle_identity: Some(oracle.relational_oracle_identity().clone()),
            bridge_oracle_identity: None,
            verification_identity,
            component_count,
        }
    }

    pub(crate) fn bridge(
        execution_subject_identity: &WorthQueryEvidenceIdentity,
        oracle: &BridgeExecutionOracle,
    ) -> Self {
        let verification_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_oracle_verification_v1",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("kind"),
                    EffectExecutionOracleVerificationKind::Writeback.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("execution_subject"),
                    execution_subject_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("bridge_oracle"),
                    oracle.bridge_oracle_identity(),
                )
                .field_usize(WorthQueryEvidenceTag::new("component_count"), 1)
                .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            verification_kind: EffectExecutionOracleVerificationKind::Writeback,
            execution_subject_identity: execution_subject_identity.clone(),
            relational_oracle_identity: None,
            bridge_oracle_identity: Some(oracle.bridge_oracle_identity().clone()),
            verification_identity,
            component_count: 1,
        }
    }

    pub fn verification_kind(&self) -> EffectExecutionOracleVerificationKind {
        self.verification_kind
    }

    pub fn execution_subject_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.execution_subject_identity
    }

    pub fn execution_subject_for_reporting(&self) -> &str {
        self.execution_subject_identity.as_str()
    }

    pub fn relational_oracle_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.relational_oracle_identity.as_ref()
    }

    pub fn relational_oracle_for_reporting(&self) -> Option<&str> {
        self.relational_oracle_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn bridge_oracle_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.bridge_oracle_identity.as_ref()
    }

    pub fn bridge_oracle_for_reporting(&self) -> Option<&str> {
        self.bridge_oracle_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn verification_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.verification_identity
    }

    pub fn verification_for_reporting(&self) -> &str {
        self.verification_identity.as_str()
    }

    pub fn component_count(&self) -> usize {
        self.component_count
    }
}
