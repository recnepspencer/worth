pub(crate) use super::handles::SupplyChainSemanticHandles;
pub(crate) use super::program::CompiledSupplyChainProgram;
pub(crate) use worth_relational::facade::history::RelationalCommitReceipt;
pub(crate) use worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt;
pub(crate) use worth_relational::facade::runtime::RelationalRuntime;
pub(crate) use worth_relational::facade::transactions::CommitResult;

pub(crate) struct ProductionSeededSupplyChainWorld {
    pub(crate) runtime: RelationalRuntime,
    pub(crate) basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pub(crate) program: CompiledSupplyChainProgram,
    pub(crate) handles: SupplyChainSemanticHandles,
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) commit_result: CommitResult,
    pub(crate) schema_receipt: RelationalInitialSchemaInstallationReceipt,
}
