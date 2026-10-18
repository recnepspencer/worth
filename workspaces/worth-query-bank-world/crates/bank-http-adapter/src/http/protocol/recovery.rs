use serde::{Deserialize, Serialize};

use super::{
    BankHttpCommitDescription, BankHttpCredential, BankHttpDenial, BankHttpEffectControls,
    BankHttpProtocolVersion,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpEstateNotificationRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub estate: String,
    pub notice: String,
    pub subject: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpEstateNotificationOutcome {
    Owning {
        #[serde(rename = "operation")]
        outcome: super::BankHttpMutationOutcome,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpRecoveryRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpEffectControls,
    pub recovery: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpRecoveryPosture {
    Reversible,
    Compensatable,
    Reconcilable,
    Irreversible,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BankHttpRecoveryWork {
    pub basis_preparations: usize,
    pub digest_derivations: usize,
    pub canonical_encoded_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpRecoveryInspectionOutcome {
    Inspected {
        request_id: String,
        posture: BankHttpRecoveryPosture,
        work: BankHttpRecoveryWork,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpRecoveryStatus {
    TokenIssued,
    Completed,
    OperatorRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpRecoveryRetryDisposition {
    Retried,
    AlreadyRetried,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpRecoverySafeRetryOutcome {
    OutcomeUnknown {
        request_id: String,
        recovery: String,
    },
    Applied {
        request_id: String,
        disposition: BankHttpRecoveryRetryDisposition,
        external_completion: bool,
        fresh_attempt: bool,
    },
    /// The recovered effect already reached its one terminal completion, so no
    /// attempt was made. `commit` describes the original commit this recovery
    /// token was issued for; there is nothing to refresh or retry.
    AlreadyCompleted {
        request_id: String,
        commit: BankHttpCommitDescription,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}
