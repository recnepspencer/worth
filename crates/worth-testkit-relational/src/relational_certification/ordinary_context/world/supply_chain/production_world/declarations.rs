pub use crate::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles;
pub use crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram;
pub use worth_relational::facade::history::RelationalCommitReceipt;
pub use worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt;
pub use worth_relational::facade::runtime::RelationalRuntime;
pub use worth_relational::facade::transactions::CommitResult;

pub struct ProductionSeededSupplyChainWorld {
    pub runtime: RelationalRuntime,
    pub basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pub program: CompiledSupplyChainProgram,
    pub handles: SupplyChainSemanticHandles,
    pub commit: RelationalCommitReceipt,
    pub commit_result: CommitResult,
    pub schema_receipt: RelationalInitialSchemaInstallationReceipt,
}
