use bank_http_adapter::{
    BankHttpAccountActivityPageOutcome, BankHttpAccountSummaryOutcome,
    BankHttpEstateDisbursementOutcome, BankHttpEstateNotificationOutcome, BankHttpMutationControls,
    BankHttpMutationOperation, BankHttpMutationOutcome, BankHttpRecoveryInspectionOutcome,
    BankHttpRequestControls,
};
use serde::{Deserialize, Serialize};

mod elevation;
pub use elevation::{
    BankUserNodeElevationApprovalOutcome, BankUserNodeElevationApprovalRequest,
    BankUserNodeElevationRequest, BankUserNodeElevationRequestOutcome,
    BankUserNodeElevationRevocationOutcome, BankUserNodeElevationRevocationRequest,
    BankUserNodeMandatoryReviewOutcome, BankUserNodeMandatoryReviewRequest,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeAccountSummaryRequest {
    pub request_id: String,
    pub controls: BankHttpRequestControls,
    pub account: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeAccountActivityStreamRequest {
    pub request_id: String,
    pub controls: BankHttpRequestControls,
    pub account: String,
    pub source_buffer_capacity: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeAccountActivityPageRequest {
    pub request_id: String,
    pub controls: BankHttpRequestControls,
    pub account: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeAccountActivityResumeRequest {
    pub request_id: String,
    pub controls: BankHttpRequestControls,
    pub account: String,
    pub continuation: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BankUserNodeMutationRequest {
    pub request_id: String,
    pub controls: BankHttpMutationControls,
    pub idempotency_key: String,
    #[serde(flatten)]
    pub operation: BankHttpMutationOperation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeEstateNotificationRequest {
    pub request_id: String,
    pub controls: BankHttpMutationControls,
    pub idempotency_key: String,
    pub estate: String,
    pub notice: String,
    pub subject: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeRecoveryRequest {
    pub request_id: String,
    pub controls: BankHttpMutationControls,
    pub recovery: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeEstateDisbursementRequest {
    pub request_id: String,
    pub controls: BankHttpMutationControls,
    pub idempotency_key: String,
    pub estate: String,
    pub source_account: String,
    pub destination_account: String,
    pub beneficiary: String,
    pub amount_minor_units: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankUserNodeAuthorizationOutcome {
    AuthorizationRequired { authorization_url: String },
    Authenticated,
    Revoked,
    Denied { denial: BankUserNodeDenial },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankUserNodeDenialKind {
    MalformedRequest,
    AuthorizationAlreadyPending,
    AuthorizationNotPending,
    AuthorizationRejected,
    NoAuthenticatedSession,
    RequestSaturated,
    UpstreamDeadlineExceeded,
    UpstreamUnavailable,
    UpstreamProtocolViolation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BankUserNodeDenial {
    pub kind: BankUserNodeDenialKind,
}

impl BankUserNodeDenial {
    pub const fn new(kind: BankUserNodeDenialKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeAccountSummaryOutcome {
    Forwarded {
        response: BankHttpAccountSummaryOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeAccountActivityPageOutcome {
    Forwarded {
        response: BankHttpAccountActivityPageOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeMutationOutcome {
    Forwarded { response: BankHttpMutationOutcome },
    Denied { denial: BankUserNodeDenial },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeEstateNotificationOutcome {
    Forwarded {
        response: BankHttpEstateNotificationOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeRecoveryInspectionOutcome {
    Forwarded {
        response: BankHttpRecoveryInspectionOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "node_outcome", rename_all = "snake_case")]
pub enum BankUserNodeEstateDisbursementOutcome {
    Forwarded {
        response: BankHttpEstateDisbursementOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}
