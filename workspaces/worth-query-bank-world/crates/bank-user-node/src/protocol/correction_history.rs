use super::BankUserNodeDenial;
use bank_http_adapter::{
    BankHttpCorrectionHistoryControls, BankHttpCorrectionHistoryOutcome, BankHttpCorrectionSource,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeCorrectionHistoryRequest {
    pub request_id: String,
    pub controls: BankHttpCorrectionHistoryControls,
    pub source: BankHttpCorrectionSource,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankUserNodeCorrectionHistoryResumeRequest {
    pub request_id: String,
    pub controls: BankHttpCorrectionHistoryControls,
    pub source: BankHttpCorrectionSource,
    pub continuation: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum BankUserNodeCorrectionHistoryOutcome {
    Forwarded {
        response: BankHttpCorrectionHistoryOutcome,
    },
    Denied {
        denial: BankUserNodeDenial,
    },
}
