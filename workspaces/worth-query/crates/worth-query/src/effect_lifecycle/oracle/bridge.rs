use worth_runtime_bridge::facade::RuntimeBridge;

use super::{
    bridge_oracle::{
        bridge_observation_execution_receipt_subject_identity,
        bridge_observation_execution_record_subject_identity,
        bridge_observation_outcome_subject_identity, bridge_observation_receipt_subject_identity,
        bridge_observation_request_subject_identity, BridgeExecutionOracle,
    },
    EffectExecutionOracleError, EffectExecutionOracleErrorKind, EffectExecutionOracleVerification,
};
use crate::effect_lifecycle::execution::ExecutedEffectPlan;

impl ExecutedEffectPlan {
    pub fn verify_against_bridge_runtime(
        &self,
        runtime: &RuntimeBridge,
    ) -> Result<EffectExecutionOracleVerification, EffectExecutionOracleError> {
        let oracle = matching_bridge_oracle_for_plan(runtime, self)?;
        self.verify_against_bridge_oracle(&oracle)
    }

    pub fn verify_against_bridge_oracle(
        &self,
        oracle: &BridgeExecutionOracle,
    ) -> Result<EffectExecutionOracleVerification, EffectExecutionOracleError> {
        let Some(execution) = self.writeback_execution() else {
            return Err(EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::BridgeOracleUnsupportedEffect,
                "bridge oracle verification requires an executed writeback artifact",
                self.effect_execution_identity(),
                Some(oracle.bridge_oracle_identity()),
            ));
        };
        if bridge_observation_outcome_subject_identity(execution.outcome().digest())
            != *oracle.outcome_subject_identity()
            || bridge_observation_outcome_subject_identity(
                execution.execution_receipt().authority_outcome_digest(),
            ) != *oracle.outcome_subject_identity()
        {
            return Err(EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::BridgeOracleOutcomeMismatch,
                format!(
                    "bridge oracle observed outcome `{}` but executed writeback produced `{}` / `{}`",
                    oracle.outcome_for_reporting(),
                    execution.outcome().digest(),
                    execution.execution_receipt().authority_outcome_digest()
                ),
                self.effect_execution_identity(),
                Some(oracle.bridge_oracle_identity()),
            ));
        }
        if execution.outcome().outcome_class() != oracle.outcome_class()
            || execution.authority_receipt().outcome_class() != oracle.outcome_class()
        {
            return Err(EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::BridgeOracleOutcomeMismatch,
                format!(
                    "bridge oracle observed outcome class `{:?}` but executed writeback produced `{:?}` / `{:?}`",
                    oracle.outcome_class(),
                    execution.outcome().outcome_class(),
                    execution.authority_receipt().outcome_class()
                ),
                self.effect_execution_identity(),
                Some(oracle.bridge_oracle_identity()),
            ));
        }
        if bridge_observation_receipt_subject_identity(execution.authority_receipt().digest())
            != *oracle.receipt_subject_identity()
            || bridge_observation_receipt_subject_identity(
                execution.execution_receipt().authority_receipt_digest(),
            ) != *oracle.receipt_subject_identity()
        {
            return Err(EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::BridgeOracleReceiptMismatch,
                format!(
                    "bridge oracle observed receipt `{}` but executed writeback produced `{}` / `{}`",
                    oracle.receipt_for_reporting(),
                    execution.authority_receipt().digest(),
                    execution.execution_receipt().authority_receipt_digest()
                ),
                self.effect_execution_identity(),
                Some(oracle.bridge_oracle_identity()),
            ));
        }
        if bridge_observation_request_subject_identity(
            execution.authority_receipt().request_digest(),
        ) != *oracle.request_subject_identity()
        {
            return Err(EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::BridgeOracleRequestMismatch,
                format!(
                    "bridge oracle observed request `{}` but executed writeback produced `{}`",
                    oracle.request_for_reporting(),
                    execution.authority_receipt().request_digest()
                ),
                self.effect_execution_identity(),
                Some(oracle.bridge_oracle_identity()),
            ));
        }
        if let Some(execution_receipt_subject_identity) =
            oracle.execution_receipt_subject_identity()
            && bridge_observation_execution_receipt_subject_identity(
                execution.execution_receipt().digest(),
            ) != *execution_receipt_subject_identity
        {
            return Err(EffectExecutionOracleError::new(
                    EffectExecutionOracleErrorKind::BridgeOracleReceiptMismatch,
                    format!(
                        "bridge oracle observed execution receipt `{}` but executed writeback produced `{}`",
                        execution_receipt_subject_identity.as_str(),
                        execution.execution_receipt().digest()
                    ),
                    self.effect_execution_identity(),
                    Some(oracle.bridge_oracle_identity()),
                ));
        }
        Ok(EffectExecutionOracleVerification::bridge(
            self.effect_execution_identity(),
            oracle,
        ))
    }
}

fn matching_bridge_oracle_for_plan(
    runtime: &RuntimeBridge,
    executed: &ExecutedEffectPlan,
) -> Result<BridgeExecutionOracle, EffectExecutionOracleError> {
    let Some(execution) = executed.writeback_execution() else {
        return Err(EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::BridgeOracleUnsupportedEffect,
            "bridge oracle verification requires an executed writeback artifact",
            executed.effect_execution_identity(),
            None,
        ));
    };
    let record = runtime
        .diagnostics()
        .writeback_execution_records()
        .into_iter()
        .find(|record| {
            record.outcome_digest() == Some(execution.outcome().digest())
                && record.request_digest() == Some(execution.authority_receipt().request_digest())
                && record.receipt_digest() == Some(execution.authority_receipt().digest())
        })
        .ok_or_else(|| {
            EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::BridgeObservationMissingWritebackRecord,
                "independent bridge oracle inspection could not find a matching writeback execution record",
                executed.effect_execution_identity(),
                None,
            )
        })?;
    let outcome_digest = record.outcome_digest().ok_or_else(|| {
        EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::BridgeObservationIncompleteWritebackRecord,
            format!(
                "matching bridge writeback record `{}` is missing outcome digest",
                record.digest()
            ),
            executed.effect_execution_identity(),
            Some(&bridge_observation_execution_record_subject_identity(
                record.digest(),
            )),
        )
    })?;
    let outcome_class = record.outcome_class().ok_or_else(|| {
        EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::BridgeObservationIncompleteWritebackRecord,
            format!(
                "matching bridge writeback record `{}` is missing outcome class",
                record.digest()
            ),
            executed.effect_execution_identity(),
            Some(&bridge_observation_execution_record_subject_identity(
                record.digest(),
            )),
        )
    })?;
    let request_digest = record.request_digest().ok_or_else(|| {
        EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::BridgeObservationIncompleteWritebackRecord,
            format!(
                "matching bridge writeback record `{}` is missing request digest",
                record.digest()
            ),
            executed.effect_execution_identity(),
            Some(&bridge_observation_execution_record_subject_identity(
                record.digest(),
            )),
        )
    })?;
    let receipt_digest = record.receipt_digest().ok_or_else(|| {
        EffectExecutionOracleError::new(
            EffectExecutionOracleErrorKind::BridgeObservationIncompleteWritebackRecord,
            format!(
                "matching bridge writeback record `{}` is missing receipt digest",
                record.digest()
            ),
            executed.effect_execution_identity(),
            Some(&bridge_observation_execution_record_subject_identity(
                record.digest(),
            )),
        )
    })?;
    let oracle = BridgeExecutionOracle::new(
        bridge_observation_execution_record_subject_identity(record.digest()),
        bridge_observation_outcome_subject_identity(outcome_digest),
        outcome_class,
        bridge_observation_request_subject_identity(request_digest),
        bridge_observation_receipt_subject_identity(receipt_digest),
    );
    Ok(match record.execution_receipt_digest() {
        Some(execution_receipt_digest) => oracle.with_execution_receipt_subject_identity(
            bridge_observation_execution_receipt_subject_identity(execution_receipt_digest),
        ),
        None => oracle,
    })
}
