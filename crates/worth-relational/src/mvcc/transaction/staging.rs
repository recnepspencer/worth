#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalTransactionStagingDenial {
    OverlayCapacityExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    FootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointCapacityExhausted {
        maximum_savepoints: usize,
    },
    SavepointFootprintCapacityExhausted {
        maximum_loci: usize,
        required_loci: usize,
    },
    SavepointIdentityExhausted,
    MaterializationAuthorityRequired,
    MaterializationModeMismatch,
}

impl RelationalTransactionStagingDenial {
    pub(crate) fn into_conflict(self) -> crate::transactions::data::CommitConflict {
        let class = match self {
            Self::FootprintCapacityExhausted {
                maximum_loci,
                required_loci,
            } => crate::transactions::data::ConflictClass::TransactionFootprintBudgetExceeded {
                maximum_loci,
                required_loci,
            },
            Self::OverlayCapacityExhausted {
                maximum_bytes,
                required_bytes,
            } => crate::transactions::data::ConflictClass::TransactionOverlayBudgetExceeded {
                maximum_bytes,
                required_bytes,
            },
            Self::SavepointCapacityExhausted { maximum_savepoints } => {
                crate::transactions::data::ConflictClass::TransactionSavepointBudgetExceeded {
                    maximum_savepoints,
                }
            }
            Self::SavepointFootprintCapacityExhausted {
                maximum_loci,
                required_loci,
            } => crate::transactions::data::ConflictClass::TransactionSavepointFootprintBudgetExceeded {
                maximum_loci,
                required_loci,
            },
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
