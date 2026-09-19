use serde::{Deserialize, Serialize};

use super::{
    BankHttpCommitDescription, BankHttpCommitDisposition, BankHttpCredential, BankHttpDenial,
    BankHttpMutationControls, BankHttpProtocolVersion,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpEstateNotificationRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpMutationControls,
    pub idempotency_key: String,
    pub estate: String,
    pub notice: String,
    pub subject: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpEstateNotificationOutcome {
    Applied {
        request_id: String,
        disposition: BankHttpCommitDisposition,
        commit: BankHttpCommitDescription,
        recovery: Option<String>,
        recovery_status: BankHttpRecoveryStatus,
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
    pub controls: BankHttpMutationControls,
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
    Applied {
        request_id: String,
        disposition: BankHttpRecoveryRetryDisposition,
        external_completion: bool,
        fresh_attempt: bool,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}
