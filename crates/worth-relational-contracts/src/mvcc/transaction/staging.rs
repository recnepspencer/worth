use crate::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalTransactionStagingDenial {
    AllocationDenied(worth_execution::ExecutionAllocationDenial),
    CardinalityOverflow,
    SymbolIdentityExhausted,
    InputDirectoryAllocationDenied { requested_batches: usize },
    SavepointCapacityExhausted { maximum_savepoints: usize },
    SavepointIdentityExhausted,
    MaterializationAuthorityRequired,
    MaterializationModeMismatch,
}

impl From<worth_execution::ExecutionAllocationDenial> for RelationalTransactionStagingDenial {
    fn from(denial: worth_execution::ExecutionAllocationDenial) -> Self {
        Self::AllocationDenied(denial)
    }
}
impl RelationalTransactionStagingDenial {
    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match self {
            Self::AllocationDenied(denial) => Some(denial),
            _ => None,
        }
    }
}

impl From<crate::symbols::data::RelationalSymbolExhausted> for RelationalTransactionStagingDenial {
    fn from(_: crate::symbols::data::RelationalSymbolExhausted) -> Self {
        Self::SymbolIdentityExhausted
    }
}

/// Operations used by the Relational owners.
pub trait RelationalTransactionStagingDenialOwnerPort: Sized {
    fn into_conflict(self) -> crate::transactions::data::outcomes::commit_conflict::CommitConflict;
}

impl RelationalTransactionStagingDenialOwnerPort for RelationalTransactionStagingDenial {
    fn into_conflict(self) -> crate::transactions::data::outcomes::commit_conflict::CommitConflict {
        let class = match self {
            Self::SymbolIdentityExhausted => crate::transactions::data::outcomes::conflict_class::ConflictClass::SymbolIdentityExhausted,
            Self::AllocationDenied(denial) => crate::transactions::data::outcomes::conflict_class::ConflictClass::ExecutionAllocationDenied { denial },
            Self::CardinalityOverflow => crate::transactions::data::outcomes::conflict_class::ConflictClass::TransactionStagingCardinalityOverflow,
            Self::InputDirectoryAllocationDenied { requested_batches } => crate::transactions::data::outcomes::conflict_class::ConflictClass::TransactionInputDirectoryAllocationDenied { requested_batches },
            Self::SavepointCapacityExhausted { maximum_savepoints } => {
                crate::transactions::data::outcomes::conflict_class::ConflictClass::TransactionSavepointBudgetExceeded {
                    maximum_savepoints,
                }
            }
            Self::SavepointIdentityExhausted => {
                crate::transactions::data::outcomes::conflict_class::ConflictClass::TransactionSavepointIdentityExhausted
            }
            Self::MaterializationAuthorityRequired => {
                crate::transactions::data::outcomes::conflict_class::ConflictClass::MaterializationAuthorityRequired
            }
            Self::MaterializationModeMismatch => {
                crate::transactions::data::outcomes::conflict_class::ConflictClass::MaterializationModeMismatch
            }
        };
        crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(class)
    }
}
