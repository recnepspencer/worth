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
        denial: BankHttpDenial,
    },
}
