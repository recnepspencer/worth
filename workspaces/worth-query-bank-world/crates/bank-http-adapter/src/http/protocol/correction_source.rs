//! Descriptive occurrence coordinates carried by correction commands and results.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpCorrectionSource {
    pub world_owner: u64,
    pub world_commit: u64,
    pub relational: BankHttpCorrectionCommit,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BankHttpCorrectionCommit {
    pub commit_id: u64,
    pub version_id: u64,
    pub branch_id: String,
    pub parents: Vec<u64>,
}
