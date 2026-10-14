//! Current owning money command wire; custody identifiers carry no permission.
use super::{
    BankHttpAccountActivity, BankHttpCredential, BankHttpDenial, BankHttpProtocolVersion,
    BankHttpQueryPublication,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpMutationControls {
    pub deadline_milliseconds: u64,
    pub maximum_history_work: usize,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpMutationReadControls {
    pub deadline_milliseconds: u64,
    pub maximum_results: usize,
    pub maximum_work: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "command", rename_all = "snake_case")]
pub enum BankHttpMutationRequest {
    Create {
        protocol: BankHttpProtocolVersion,
        request_id: String,
        credential: BankHttpCredential,
        custody_id: String,
        sequence: u64,
        controls: BankHttpMutationControls,
        idempotency_key: String,
        operation: BankHttpMutationOperation,
    },
    Resume {
        protocol: BankHttpProtocolVersion,
        request_id: String,
        credential: BankHttpCredential,
        custody_id: String,
        sequence: u64,
        controls: BankHttpMutationControls,
    },
    Read {
        protocol: BankHttpProtocolVersion,
        request_id: String,
        credential: BankHttpCredential,
        custody_id: String,
        sequence: u64,
        controls: BankHttpMutationReadControls,
    },
    Close {
        protocol: BankHttpProtocolVersion,
        request_id: String,
        credential: BankHttpCredential,
        custody_id: String,
        sequence: u64,
        controls: super::BankHttpEffectControls,
    },
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
        expected_accounting_revision: Option<u64>,
        expected_status: Option<BankHttpExpectedAccountStatus>,
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpMutationOutcome {
    Rejected {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
    OutcomeUnknown {
        request_id: String,
        custody_id: String,
    },
    Pending {
        request_id: String,
        custody_id: String,
        sequence: u64,
        posture: BankHttpMutationPosture,
    },
    Refused {
        request_id: String,
        custody_id: String,
        sequence: u64,
        denial: BankHttpDenial,
        accepted_sequence: Option<u64>,
    },
    Completed {
        request_id: String,
        custody_id: String,
        sequence: u64,
        recovered: bool,
        commit: BankHttpCommitDescription,
    },
    Terminal {
        request_id: String,
        custody_id: String,
        sequence: u64,
    },
    Delivered {
        request_id: String,
        custody_id: String,
        sequence: u64,
        activity: BankHttpAccountActivity,
        publication: BankHttpQueryPublication,
    },
    Closed {
        request_id: String,
        custody_id: String,
        sequence: u64,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpExpectedAccountStatus {
    Open,
    Frozen,
    Closed,
}
impl From<BankHttpExpectedAccountStatus> for bank_domain::schema::AccountStatus {
    fn from(value: BankHttpExpectedAccountStatus) -> Self {
        match value {
            BankHttpExpectedAccountStatus::Open => Self::Open,
            BankHttpExpectedAccountStatus::Frozen => Self::Frozen,
            BankHttpExpectedAccountStatus::Closed => Self::Closed,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BankHttpMutationPosture {
    Initial,
    ProductUnpublished,
    RecoveryPerformed,
    RetainedDeliveryPending,
    ProvedAborted,
    RetryableNoEffect,
    Indeterminate,
    Terminal,
}
impl From<worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture>
    for BankHttpMutationPosture
{
    fn from(
        value: worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture,
    ) -> Self {
        use worth_query_host::facade::application_entry::WorthQuerySourceOnlyPosture as Native;
        match value {
            Native::Initial => Self::Initial,
            Native::ProductUnpublished => Self::ProductUnpublished,
            Native::RecoveryPerformed => Self::RecoveryPerformed,
            Native::RetainedDeliveryPending => Self::RetainedDeliveryPending,
            Native::ProvedAborted => Self::ProvedAborted,
            Native::RetryableNoEffect => Self::RetryableNoEffect,
            Native::Indeterminate => Self::Indeterminate,
            Native::Terminal => Self::Terminal,
        }
    }
}
