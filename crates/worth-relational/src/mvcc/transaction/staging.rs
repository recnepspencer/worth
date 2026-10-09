#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalTransactionStagingDenial {
    AllocationDenied(worth_execution::ExecutionAllocationDenial),
    CardinalityOverflow,
    InputDirectoryAllocationDenied { requested_batches: usize },
    SavepointCapacityExhausted { maximum_savepoints: usize },
    SavepointIdentityExhausted,
    MaterializationAuthorityRequired,
    MaterializationModeMismatch,
}

impl RelationalTransactionStagingDenial {
    pub(crate) fn into_conflict(self) -> crate::transactions::data::CommitConflict {
        let class = match self {
            Self::AllocationDenied(denial) => crate::transactions::data::ConflictClass::ExecutionAllocationDenied { denial },
            Self::CardinalityOverflow => crate::transactions::data::ConflictClass::TransactionStagingCardinalityOverflow,
            Self::InputDirectoryAllocationDenied { requested_batches } => crate::transactions::data::ConflictClass::TransactionInputDirectoryAllocationDenied { requested_batches },
            Self::SavepointCapacityExhausted { maximum_savepoints } => {
                crate::transactions::data::ConflictClass::TransactionSavepointBudgetExceeded {
                    maximum_savepoints,
                }
            }
            Self::SavepointIdentityExhausted => {
                crate::transactions::data::ConflictClass::TransactionSavepointIdentityExhausted
            }
            Self::MaterializationAuthorityRequired => {
                crate::transactions::data::ConflictClass::MaterializationAuthorityRequired
            }
            Self::MaterializationModeMismatch => {
                crate::transactions::data::ConflictClass::MaterializationModeMismatch
            }
        };
        crate::transactions::data::CommitConflict::new(class)
    }
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
