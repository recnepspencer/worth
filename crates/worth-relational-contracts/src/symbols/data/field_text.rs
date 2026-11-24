//! Typed refusals for admission of Native field text.
#[derive(Debug)]
pub enum RelationalFieldTextAdmissionDenial {
    InvalidUtf8,
    StorageUnavailable(crate::storage::data::storage_unavailable::RelationalStorageUnavailable),
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    InvalidSymbolDictionary,
    ForeignBasis,
    StaleBasis,
    SymbolExhausted,
    Stopped(worth_execution::MapKernelStop),
    Allocation(worth_execution::ExecutionAllocationDenial),
}

pub fn field_text_recovery_stop(
    denial: RelationalFieldTextAdmissionDenial,
) -> crate::durability::data::recovery_stop::RelationalRecoveryStop {
    use crate::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
    use crate::durability::data::recovery_stop::RelationalRecoveryStop;
    match denial {
        RelationalFieldTextAdmissionDenial::StorageUnavailable(failure) => failure.into(),
        RelationalFieldTextAdmissionDenial::Request(cause) => {
            RelationalRecoveryStop::Request(cause)
        }
        RelationalFieldTextAdmissionDenial::Allocation(allocation) => match allocation.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
                RelationalRecoveryStop::Request(cause.into())
            }
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
                RelationalRecoveryStop::Allocation(allocation)
            }
        },
        RelationalFieldTextAdmissionDenial::Stopped(stop) => RelationalRecoveryStop::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::from(stop),
        ),
        other => DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("invalid managed native text: {other:?}"),
        )
        .into(),
    }
}

impl std::fmt::Display for RelationalFieldTextAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StorageUnavailable(failure) => {
                write!(formatter, "native text storage unavailable: {failure:?}")
            }
            Self::Request(cause) => write!(formatter, "native text request refused: {cause:?}"),
            Self::Allocation(cause) => write!(formatter, "native field text allocation: {cause}"),
            Self::Stopped(cause) => write!(formatter, "native field text request: {cause:?}"),
            Self::InvalidUtf8 => formatter.write_str("native field text requires UTF-8"),
            Self::InvalidSymbolDictionary => formatter
                .write_str("native symbol dictionary has conflicting or exhausted meanings"),
            Self::ForeignBasis => formatter.write_str("native field text belongs to another basis"),
            Self::StaleBasis => formatter.write_str("native field text basis is no longer current"),
            Self::SymbolExhausted => formatter.write_str("native symbol identity is exhausted"),
        }
    }
}
impl std::error::Error for RelationalFieldTextAdmissionDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(cause) => Some(cause),
            _ => None,
        }
    }
}
