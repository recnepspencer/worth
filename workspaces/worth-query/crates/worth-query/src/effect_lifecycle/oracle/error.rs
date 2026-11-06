use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectExecutionOracleErrorKind {
    RelationalObservationMissingBranchHead,
    RelationalOracleUnsupportedEffect,
    RelationalOracleBranchMismatch,
    RelationalOracleCommitMismatch,
    BridgeObservationMissingWritebackRecord,
    BridgeObservationIncompleteWritebackRecord,
    BridgeOracleUnsupportedEffect,
    BridgeOracleOutcomeMismatch,
    BridgeOracleReceiptMismatch,
    BridgeOracleRequestMismatch,
    BatchOracleMixedTargetBranch,
}

impl EffectExecutionOracleErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RelationalObservationMissingBranchHead => {
                "relational_observation_missing_branch_head"
            }
            Self::RelationalOracleUnsupportedEffect => "relational_oracle_unsupported_effect",
            Self::RelationalOracleBranchMismatch => "relational_oracle_branch_mismatch",
            Self::RelationalOracleCommitMismatch => "relational_oracle_commit_mismatch",
            Self::BridgeObservationMissingWritebackRecord => {
                "bridge_observation_missing_writeback_record"
            }
            Self::BridgeObservationIncompleteWritebackRecord => {
                "bridge_observation_incomplete_writeback_record"
            }
            Self::BridgeOracleUnsupportedEffect => "bridge_oracle_unsupported_effect",
            Self::BridgeOracleOutcomeMismatch => "bridge_oracle_outcome_mismatch",
            Self::BridgeOracleReceiptMismatch => "bridge_oracle_receipt_mismatch",
            Self::BridgeOracleRequestMismatch => "bridge_oracle_request_mismatch",
            Self::BatchOracleMixedTargetBranch => "batch_oracle_mixed_target_branch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectExecutionOracleError {
    kind: EffectExecutionOracleErrorKind,
    message: String,
    execution_subject_identity: WorthQueryEvidenceIdentity,
    oracle_identity: Option<WorthQueryEvidenceIdentity>,
    error_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
}

impl EffectExecutionOracleError {
    pub(crate) fn new(
        kind: EffectExecutionOracleErrorKind,
        message: impl Into<String>,
        execution_subject_identity: &WorthQueryEvidenceIdentity,
        oracle_identity: Option<&WorthQueryEvidenceIdentity>,
    ) -> Self {
        let message = message.into();
        let error_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_oracle_error_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("execution_subject"),
                    execution_subject_identity,
                )
                .optional_evidence_identity(WorthQueryEvidenceTag::new("oracle"), oracle_identity)
                .field_shape(WorthQueryEvidenceTag::new("message"), message.as_str())
                .seal();
        Self {
            kind,
            message,
            execution_subject_identity: execution_subject_identity.clone(),
            oracle_identity: oracle_identity.cloned(),
            error_identity,
        }
    }

    pub fn kind(&self) -> EffectExecutionOracleErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn execution_subject_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.execution_subject_identity
    }

    pub fn execution_subject_for_reporting(&self) -> &str {
        self.execution_subject_identity.as_str()
    }

    pub fn oracle_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.oracle_identity.as_ref()
    }

    pub fn oracle_for_reporting(&self) -> Option<&str> {
        self.oracle_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn error_identity(&self) -> Result<&WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.error_identity.as_ref().map_err(|cause| *cause)
    }

    pub fn error_for_reporting(&self) -> Result<&str, IdentityDerivationDenial> {
        Ok(self.error_identity()?.as_str())
    }
}

pub(crate) fn bridge_oracle_observation_subject() -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "bridge_oracle_observation_subject_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), "last_writeback")
        .seal()
        .expect("private bridge observation descriptor contains two fixed literal entries")
}
