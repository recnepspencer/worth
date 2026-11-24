use super::index_access_parity::{IndexParityMode, QueryAccessPath};
use super::query_outcome::QueryExecutionOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexParityVerifiedQueryOutcome {
    pub execution: QueryExecutionOutcome,
    pub access_path: QueryAccessPath,
    pub parity_mode: IndexParityMode,
    pub parity_basis_digest: String,
}
