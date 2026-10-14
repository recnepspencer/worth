use serde::{Deserialize, Serialize};

use super::{
    BankHttpCommitDisposition, BankHttpCredential, BankHttpDenial, BankHttpEffectControls,
    BankHttpProtocolVersion,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpEmergencyAccessReason {
    PreventImmediateLoss,
    ProtectVulnerableCustomer,
    MeetLegalDeadline,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpRestrictedBankField {
    CustomerIdentity,
    BeneficiaryIdentity,
    LegalDocument,
    AccountDetails,
    PostingHistory,
    AuditTrail,
    GovernanceMetadata,
    EmergencyAccessActivity,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpElevationClosureKind {
    Revoked,
    Expired,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpElevationRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
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
pub struct BankHttpElevationApprovalRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub elevation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpElevationRevocationRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub elevation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpMandatoryReviewRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpEffectControls,
    pub idempotency_key: String,
    pub mandatory_review: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpElevationRequestOutcome {
    Requested {
        request_id: String,
        disposition: BankHttpCommitDisposition,
        elevation: String,
        changed_record_count: usize,
        emitted_effect_count: usize,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpElevationApprovalOutcome {
    Approved {
        request_id: String,
        disposition: BankHttpCommitDisposition,
        elevation: String,
        changed_record_count: usize,
        emitted_effect_count: usize,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpElevationRevocationOutcome {
    Closed {
        request_id: String,
        disposition: BankHttpCommitDisposition,
        mandatory_review: String,
        closure: BankHttpElevationClosureKind,
        changed_record_count: usize,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpMandatoryReviewOutcome {
    Reviewed {
        request_id: String,
        disposition: BankHttpCommitDisposition,
        closure: BankHttpElevationClosureKind,
        changed_record_count: usize,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}
