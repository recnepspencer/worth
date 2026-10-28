//! Native text custody is data, never a permission or workflow grant.
use super::{StoredSymbolText, Symbol};
use crate::branch::AdmittedRelationalBranchBasis;
use worth_foundational::facade::{AspectValue, InternedString};
#[derive(Debug)]
pub enum RelationalFieldTextAdmissionDenial {
    InvalidUtf8,
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
    Request(crate::execution::RelationalExecutionDenialCause),
    InvalidSymbolDictionary,
    ForeignBasis,
    StaleBasis,
    SymbolExhausted,
    Stopped(worth_execution::MapKernelStop),
    Allocation(worth_execution::ExecutionAllocationDenial),
}
#[derive(Clone, Debug)]
pub struct RelationalFieldText {
    pub(crate) basis: AdmittedRelationalBranchBasis,
    pub(crate) symbol: Symbol,
    pub(crate) text: StoredSymbolText,
}
impl RelationalFieldText {
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
    pub fn charged_payload_bytes(&self) -> Option<u64> {
        self.text.charged_payload_bytes()
    }
    pub fn field_value_for(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<AspectValue, RelationalFieldTextAdmissionDenial> {
        if self.basis.descriptor() != basis.descriptor() {
            return Err(RelationalFieldTextAdmissionDenial::ForeignBasis);
        }
        if !basis.is_current() {
            return Err(RelationalFieldTextAdmissionDenial::StaleBasis);
        }
        Ok(AspectValue::String(InternedString::Symbol(
            worth_foundational::facade::Symbol(self.symbol.0),
        )))
    }
}

pub(crate) fn field_text_recovery_stop(
    denial: RelationalFieldTextAdmissionDenial,
) -> crate::durability::data::RelationalRecoveryStop {
    use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
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
            crate::execution::RelationalExecutionDenialCause::from(stop),
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
