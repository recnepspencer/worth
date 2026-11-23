//! Inputs selected for one admitted aggregate consequence phase.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::config::data::StorageConfig;
use crate::execution::request_work::RequestWorkBudget;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub(crate) struct AggregateDraftInputs<'a> {
    pub(crate) work: Option<&'a RequestWorkBudget>,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
    pub(crate) symbols: &'a RuntimeSymbolTable,
    pub(crate) storage: &'a StorageConfig,
}

pub(crate) struct AggregateDraftAdmission<'a> {
    pub(crate) schema: &'a RelationalBranchRootSchemaAuthority,
    pub(crate) basis: &'a AdmittedRelationalBranchBasis,
    pub(crate) materialization_only: bool,
}
