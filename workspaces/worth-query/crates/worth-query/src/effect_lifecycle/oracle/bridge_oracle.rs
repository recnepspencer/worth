use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::{BridgeWritebackOutcomeClass, RuntimeBridge};

use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::error::{
    bridge_oracle_observation_subject, EffectExecutionOracleError, EffectExecutionOracleErrorKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeExecutionOracle {
    execution_record_identity: WorthQueryEvidenceIdentity,
    outcome_subject_identity: WorthQueryEvidenceIdentity,
    outcome_class: BridgeWritebackOutcomeClass,
    request_subject_identity: WorthQueryEvidenceIdentity,
    receipt_subject_identity: WorthQueryEvidenceIdentity,
    execution_receipt_subject_identity: Option<WorthQueryEvidenceIdentity>,
    bridge_oracle_identity: WorthQueryEvidenceIdentity,
}

impl BridgeExecutionOracle {
    pub fn new(
        execution_record_identity: WorthQueryEvidenceIdentity,
        outcome_subject_identity: WorthQueryEvidenceIdentity,
        outcome_class: BridgeWritebackOutcomeClass,
        request_subject_identity: WorthQueryEvidenceIdentity,
        receipt_subject_identity: WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let bridge_oracle_identity = compose_bridge_oracle_identity(
            &execution_record_identity,
            &outcome_subject_identity,
            outcome_class,
            &request_subject_identity,
            &receipt_subject_identity,
            None,
        )?;
        Ok(Self {
            execution_record_identity,
            outcome_subject_identity,
            outcome_class,
            request_subject_identity,
            receipt_subject_identity,
            execution_receipt_subject_identity: None,
            bridge_oracle_identity,
        })
    }

    pub fn with_execution_receipt_subject_identity(
        mut self,
        execution_receipt_subject_identity: WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        self.bridge_oracle_identity = compose_bridge_oracle_identity(
            &self.execution_record_identity,
            &self.outcome_subject_identity,
            self.outcome_class,
            &self.request_subject_identity,
            &self.receipt_subject_identity,
            Some(&execution_receipt_subject_identity),
        )?;
        self.execution_receipt_subject_identity = Some(execution_receipt_subject_identity);
        Ok(self)
    }

    pub fn observe_last_writeback(
        runtime: &RuntimeBridge,
    ) -> Result<Self, EffectExecutionOracleError> {
        let record = runtime
            .diagnostics()
            .last_writeback_execution_record()
            .map_err(|absence| {
                EffectExecutionOracleError::new(
                    EffectExecutionOracleErrorKind::BridgeObservationMissingWritebackRecord,
                    format!("independent bridge oracle inspection could not find a writeback execution record: {absence:?}"),
                    &bridge_oracle_observation_subject(),
                    None,
                )
            })?;
        let outcome_digest = record
            .outcome_digest()
            .ok_or_else(|| incomplete_bridge_record_error("outcome", record.digest()))?;
        let request_digest = record
            .request_digest()
            .ok_or_else(|| incomplete_bridge_record_error("request", record.digest()))?;
        let receipt_digest = record
            .receipt_digest()
            .ok_or_else(|| incomplete_bridge_record_error("receipt", record.digest()))?;
        let execution_record_identity =
            bridge_observation_execution_record_subject_identity(record.digest())?;
        let oracle = Self::new(
            execution_record_identity,
            bridge_observation_outcome_subject_identity(outcome_digest)?,
            record
                .outcome_class()
                .ok_or_else(|| incomplete_bridge_record_error("outcome_class", record.digest()))?,
            bridge_observation_request_subject_identity(request_digest)?,
            bridge_observation_receipt_subject_identity(receipt_digest)?,
        )?;
        Ok(match record.execution_receipt_digest() {
            Some(execution_receipt_digest) => oracle.with_execution_receipt_subject_identity(
                bridge_observation_execution_receipt_subject_identity(execution_receipt_digest)?,
            )?,
            None => oracle,
        })
    }

    pub fn execution_record_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.execution_record_identity
    }

    pub fn execution_record_for_reporting(&self) -> &str {
        self.execution_record_identity.as_str()
    }

    pub fn outcome_subject_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.outcome_subject_identity
    }

    pub fn outcome_for_reporting(&self) -> &str {
        self.outcome_subject_identity.as_str()
    }

    pub fn outcome_class(&self) -> BridgeWritebackOutcomeClass {
        self.outcome_class
    }

    pub fn request_subject_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_subject_identity
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_subject_identity.as_str()
    }

    pub fn receipt_subject_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_subject_identity
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_subject_identity.as_str()
    }

    pub fn execution_receipt_subject_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.execution_receipt_subject_identity.as_ref()
    }

    pub fn execution_receipt_for_reporting(&self) -> Option<&str> {
        self.execution_receipt_subject_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn bridge_oracle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bridge_oracle_identity
    }

    pub fn bridge_oracle_for_reporting(&self) -> &str {
        self.bridge_oracle_identity.as_str()
    }
}

pub fn bridge_observation_execution_record_subject_identity(
    record_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(bridge_observation_subject_identity(
        "execution_record",
        record_digest,
    )?)
}

pub fn bridge_observation_outcome_subject_identity(
    outcome_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(bridge_observation_subject_identity(
        "outcome",
        outcome_digest,
    )?)
}

pub fn bridge_observation_request_subject_identity(
    request_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(bridge_observation_subject_identity(
        "request",
        request_digest,
    )?)
}

pub fn bridge_observation_receipt_subject_identity(
    receipt_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(bridge_observation_subject_identity(
        "receipt",
        receipt_digest,
    )?)
}

pub fn bridge_observation_execution_receipt_subject_identity(
    execution_receipt_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(bridge_observation_subject_identity(
        "execution_receipt",
        execution_receipt_digest,
    )?)
}

fn bridge_observation_subject_identity(
    kind: &str,
    observed_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "bridge_observation_subject_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind)
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("observed"),
                &WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                    .field_shape(
                        WorthQueryEvidenceTag::new("identity_family"),
                        "bridge_observed_digest_v1",
                    )
                    .field_value(
                        WorthQueryEvidenceTag::new("observed_digest"),
                        observed_digest,
                    )
                    .seal()?
                    .bridge_external_identity_evidence(),
            )
            .seal()?,
    )
}

fn compose_bridge_oracle_identity(
    execution_record_identity: &WorthQueryEvidenceIdentity,
    outcome_subject_identity: &WorthQueryEvidenceIdentity,
    outcome_class: BridgeWritebackOutcomeClass,
    request_subject_identity: &WorthQueryEvidenceIdentity,
    receipt_subject_identity: &WorthQueryEvidenceIdentity,
    execution_receipt_subject_identity: Option<&WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "bridge_execution_oracle_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("execution_record"),
                execution_record_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("outcome"),
                outcome_subject_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("outcome_class"),
                writeback_outcome_class_label(outcome_class),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("request"),
                request_subject_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("receipt"),
                receipt_subject_identity,
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("execution_receipt"),
                execution_receipt_subject_identity,
            )
            .seal()?,
    )
}

fn writeback_outcome_class_label(outcome_class: BridgeWritebackOutcomeClass) -> &'static str {
    match outcome_class {
        BridgeWritebackOutcomeClass::CanonicalNoop => "canonical-noop",
        BridgeWritebackOutcomeClass::AuthoritativeCommit => "authoritative-commit",
        BridgeWritebackOutcomeClass::Rejected => "rejected",
    }
}

fn incomplete_bridge_record_error(
    missing_field: &str,
    record_digest: &str,
) -> EffectExecutionOracleError {
    (|| -> Result<EffectExecutionOracleError, IdentityDerivationDenial> {
    Ok(EffectExecutionOracleError::new(
        EffectExecutionOracleErrorKind::BridgeObservationIncompleteWritebackRecord,
        format!(
            "independent bridge oracle inspection found writeback record `{record_digest}` without `{missing_field}`"
        ),
        &bridge_oracle_observation_subject(),
        Some(&bridge_observation_execution_record_subject_identity(record_digest)?),
    ))

 })().unwrap_or_else(EffectExecutionOracleError::from)
}
