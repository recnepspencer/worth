//! Historical selectors are descriptions; the Bank resolves them under each request.
use super::{
    BankHttpCorrectionSource, BankHttpCredential, BankHttpDenial, BankHttpProtocolVersion,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpCorrectionHistoryControls {
    pub deadline_milliseconds: u64,
    pub maximum_history_work: usize,
    pub maximum_disclosure_results: usize,
    pub maximum_entries: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpCorrectionHistoryRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpCorrectionHistoryControls,
    pub source: BankHttpCorrectionSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpCorrectionHistoryResumeRequest {
    pub protocol: BankHttpProtocolVersion,
    pub request_id: String,
    pub credential: BankHttpCredential,
    pub controls: BankHttpCorrectionHistoryControls,
    pub source: BankHttpCorrectionSource,
    pub continuation: String,
}

/// One reversal's operated source, to be freshly resolved when explicitly selected.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpCorrectionAlternative {
    pub source: BankHttpCorrectionSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankHttpCorrectionHistoryOutcome {
    Delivered {
        request_id: String,
        alternatives: Vec<BankHttpCorrectionAlternative>,
        continuation: String,
        complete: bool,
    },
    Denied {
        request_id: Option<String>,
        denial: BankHttpDenial,
    },
}
