//! Inputs selected for one admitted aggregate consequence phase.
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_contracts::internal_port::config::data::sections::StorageConfig;
use worth_relational_contracts::internal_port::execution::request_work::RequestWorkBudget;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub struct AggregateDraftInputs<'a> {
    pub work: Option<&'a RequestWorkBudget>,
    pub instrumentation: &'a RuntimeInstrumentation,
    pub symbols: &'a RuntimeSymbolTable,
    pub storage: &'a StorageConfig,
}

pub struct AggregateDraftAdmission<'a> {
    pub schema: &'a RelationalBranchRootSchemaAuthority,
    pub basis: &'a AdmittedRelationalBranchBasis,
    pub materialization_only: bool,
}
