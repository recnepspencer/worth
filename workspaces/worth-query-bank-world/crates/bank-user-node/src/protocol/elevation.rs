use bank_http_adapter::{
    BankHttpEffectControls, BankHttpElevationApprovalOutcome, BankHttpElevationRequestOutcome,
    BankHttpElevationRevocationOutcome, BankHttpEmergencyAccessReason,
    BankHttpMandatoryReviewOutcome, BankHttpRestrictedBankField,
};
use serde::{Deserialize, Serialize};

use super::BankUserNodeDenial;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeElevationRequest {
    pub request_id: String,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub estate: String,
    pub access: u64,
    pub mandatory_review: u64,
    pub upper_bound_grant: u64,
    pub reason: BankHttpEmergencyAccessReason,
    pub field: BankHttpRestrictedBankField,
    pub duration_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeElevationApprovalRequest {
    pub request_id: String,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub elevation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeElevationRevocationRequest {
    pub request_id: String,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub elevation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeMandatoryReviewRequest {
    pub request_id: String,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub mandatory_review: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeElevationRequestOutcome {
    Forwarded {
        response: BankHttpElevationRequestOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeElevationApprovalOutcome {
    Forwarded {
        response: BankHttpElevationApprovalOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeElevationRevocationOutcome {
    Forwarded {
        response: BankHttpElevationRevocationOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeMandatoryReviewOutcome {
    Forwarded {
        response: BankHttpMandatoryReviewOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}
