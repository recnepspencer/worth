//! Canonical comparison distinguishes encoding failure from refused buffer custody.
use super::OracleMismatch;
use crate::backend::{record_nested, NestedStopOwner, ScopeKernelFailure};
use worth_foundational::{ExecutionPhysicalReport, ExecutionPosture, ExecutionReport};

pub(super) enum ComparisonStop {
    Encoding(OracleMismatch),
    Refused,
}
impl From<OracleMismatch> for ComparisonStop {
    fn from(cause: OracleMismatch) -> Self {
        Self::Encoding(cause)
    }
}
impl ComparisonStop {
    pub(super) fn into_scope(self) -> ScopeKernelFailure<OracleMismatch> {
        match self {
            Self::Encoding(cause) => {
                ScopeKernelFailure::Kernel(crate::backend::KernelFailure::Domain(cause))
            }
            Self::Refused => ScopeKernelFailure::Refused(Self::Refused.finish()),
        }
    }
    pub(super) fn finish(self) -> OracleMismatch {
        match self {
            Self::Encoding(cause) => cause,
            Self::Refused => {
                record_nested(
                    ExecutionReport::new(
                        ExecutionPosture::Serial,
                        0,
                        0,
                        ExecutionPhysicalReport::default(),
                    ),
                    Some(NestedStopOwner::Request),
                );
                OracleMismatch::Stop
            }
        }
    }
}
