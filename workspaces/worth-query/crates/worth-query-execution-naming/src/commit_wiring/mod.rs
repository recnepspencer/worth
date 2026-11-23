//! End-to-end execution commit wiring attaching lowered naming birth/posture
//! plans to Relational transactions and elevating sealed births into admitted
//! [`WorthQueryName`] handles.

use worth_execution::ExecutionRequest;
use worth_relational::facade::mvcc::BranchBoundRelationalTransaction;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::CommitResult;

use crate::admission::AdmittedCommitNames;
use crate::birth_plan::ExecutionNameBirthPlan;
use crate::denial::ExecutionNamingDenial;

/// Sealed Relational commit together with the persistent names minted and
/// admitted from its canonical `LineageEventRecord`s.
#[derive(Clone, Debug)]
pub struct CommittedNamedTransaction {
    commit_result: CommitResult,
    admitted_names: AdmittedCommitNames,
}

impl CommittedNamedTransaction {
    pub fn commit_result(&self) -> &CommitResult {
        &self.commit_result
    }

    pub fn admitted_names(&self) -> &AdmittedCommitNames {
        &self.admitted_names
    }

    pub fn into_parts(self) -> (CommitResult, AdmittedCommitNames) {
        (self.commit_result, self.admitted_names)
    }
}

/// Execution authority that binds a validated [`ExecutionNameBirthPlan`] onto
/// a branch-bound Relational transaction, commits through Relational's commit
/// finalization authority (which derives live/retired lineage sets from
/// Relational's own `LineageSubsystem`), and returns admitted persistent names.
pub struct ExecutionNamingCommitAuthority;

impl ExecutionNamingCommitAuthority {
    /// Attaches `birth_plan` to `transaction`, commits atomically through
    /// `runtime`, and extracts admitted persistent names from the sealed commit.
    ///
    /// Fail-closed: if Relational refuses `DuplicateBirth`,
    /// `InvalidDiscriminatorBasis`, or any posture violation, returns
    /// `Err(ExecutionNamingDenial::RelationalNaming(_))` with zero committed
    /// rows on the branch.
    pub fn commit_with_naming_plan(
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
        mut transaction: BranchBoundRelationalTransaction,
        birth_plan: ExecutionNameBirthPlan,
    ) -> Result<CommittedNamedTransaction, ExecutionNamingDenial> {
        transaction.attach_naming_plan(birth_plan.into_relational_plan());
        let commit_result = runtime
            .commit_branch_transaction(execution, transaction)
            .map_err(|err| {
                if let Some(naming) = err.lineage_name_denial_ref() {
                    ExecutionNamingDenial::RelationalNaming(naming.clone())
                } else {
                    ExecutionNamingDenial::RelationalCommit(Box::new(err))
                }
            })?;
        let admitted_names = AdmittedCommitNames::from_committed_result(runtime, &commit_result);
        Ok(CommittedNamedTransaction {
            commit_result,
            admitted_names,
        })
    }
}
