use serde::{Deserialize, Serialize};

use super::{BankHttpCredential, BankHttpDenial, BankHttpProtocolVersion};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpMutationControls {
    pub deadline_milliseconds: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BankHttpMutationRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpMutationControls,
    pub idempotency_key: String,
    #[serde(flatten)]
    pub operation: BankHttpMutationOperation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, tag = "operation", rename_all = "snake_case")]
pub enum BankHttpMutationOperation {
    Deposit {
        institution: String,
        account: String,
        amount_minor_units: i64,
    },
    Withdraw {
        institution: String,
        account: String,
        amount_minor_units: i64,
    },
    SendMoney {
        from: String,
        recipient: String,
        amount_minor_units: i64,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpCommitDisposition {
    Committed,
    AlreadyCommitted,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BankHttpCommitDescription {
    pub changed_record_count: usize,
    pub emitted_effect_count: usize,
    pub expected_version_count: usize,
    pub expected_fact_count: usize,
    pub provider_work_units: Option<usize>,
    pub invariant_work_units: Option<usize>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpMutationFailureKind {
    ProductStale,
    ProductUnpublished,
    NoEffect,
    Stale,
    Cancelled,
    TimedOut,
    DeadlineExceeded,
    InvariantViolated,
    Aborted,
    Deferred,
    SettlementDeferred,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpProviderRecoveryKind {
    CommitRecoveryRequired,
    AbortRecoveryRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpMutationOutcome {
    Applied {
        request_id: String,
        disposition: BankHttpCommitDisposition,
        commit: BankHttpCommitDescription,
    },
    NotApplied {
        request_id: Option<String>,
        failure: BankHttpMutationFailureKind,
        stale_fact_count: Option<usize>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        provider_recovery: Option<BankHttpProviderRecoveryKind>,
        denial: BankHttpDenial,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::protocol::{BankHttpDenialKind, BankHttpNextAction};

    #[test]
    fn provider_recovery_kind_survives_the_http_wire() {
        for kind in [
            BankHttpProviderRecoveryKind::CommitRecoveryRequired,
            BankHttpProviderRecoveryKind::AbortRecoveryRequired,
        ] {
            let outcome = BankHttpMutationOutcome::NotApplied {
                request_id: Some("uncertain-commit".into()),
                failure: BankHttpMutationFailureKind::Indeterminate,
                stale_fact_count: None,
                provider_recovery: Some(kind),
                denial: BankHttpDenial::new(
                    BankHttpDenialKind::Unavailable,
                    BankHttpNextAction::ContactOperator,
                ),
            };
            let wire = serde_json::to_value(&outcome).unwrap();
            assert_eq!(
                wire["provider_recovery"],
                match kind {
                    BankHttpProviderRecoveryKind::CommitRecoveryRequired => {
                        "commit_recovery_required"
                    }
                    BankHttpProviderRecoveryKind::AbortRecoveryRequired =>
                        "abort_recovery_required",
                }
            );
            assert_eq!(
                serde_json::from_value::<BankHttpMutationOutcome>(wire).unwrap(),
                outcome
            );
        }

        let ordinary = BankHttpMutationOutcome::NotApplied {
            request_id: Some("stale".into()),
            failure: BankHttpMutationFailureKind::ProductStale,
            stale_fact_count: None,
            provider_recovery: None,
            denial: BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        };
        let wire = serde_json::to_value(&ordinary).unwrap();
        assert!(wire.get("provider_recovery").is_none());
        assert_eq!(
            serde_json::from_value::<BankHttpMutationOutcome>(wire).unwrap(),
            ordinary
        );
    }
}
