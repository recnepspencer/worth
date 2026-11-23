//! Native read-refusal evidence; Query still owns its binding and scope checks.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CustomInvariantAccessDenialKind {
    ForeignBinding,
    ForeignView,
    OutsideDeclaredAccess,
    OutsidePreparedScope,
    WrongEntityKind,
    EntityUnavailable,
    MissingRequiredField,
    InvalidValue,
    RelationNotInstalled,
    NotOrbitRelation,
    InvalidOrbit,
    InvalidComponent,
    InvalidEventChain,
    SourceProgramIdentityExhausted,
    AggregateSourceUnavailable,
    AggregatePreparationRefused,
    Execution(crate::execution::denial_cause::RelationalExecutionDenialCause),
    WorkBudgetExceeded,
    Interrupted(crate::validation::custom_rule::interruption::CustomInvariantInterruption),
    MemoryBudgetExceeded,
    MemoryAdmission(#[serde(with = "super::memory_denial")] worth_execution::MemoryLimitDenial),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CustomInvariantAccessDenial {
    kind: CustomInvariantAccessDenialKind,
    subject: String,
}

impl CustomInvariantAccessDenial {
    pub fn from_native_read(
        cause: crate::validation::data::structural_walk_denial::StructuralWalkDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        use crate::validation::data::structural_read::StructuralReadError;
        use crate::validation::data::structural_walk_denial::StructuralWalkDenialKind;
        let kind = match cause {
            StructuralWalkDenialKind::Read(StructuralReadError::WorkBudgetExceeded) => {
                CustomInvariantAccessDenialKind::WorkBudgetExceeded
            }
            StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess) => {
                CustomInvariantAccessDenialKind::OutsideDeclaredAccess
            }
            StructuralWalkDenialKind::Read(StructuralReadError::RecordUnavailable) => {
                CustomInvariantAccessDenialKind::EntityUnavailable
            }
            StructuralWalkDenialKind::Interrupted(stop) => {
                CustomInvariantAccessDenialKind::Interrupted(stop)
            }
            StructuralWalkDenialKind::MemoryBudgetExceeded => {
                CustomInvariantAccessDenialKind::MemoryBudgetExceeded
            }
            StructuralWalkDenialKind::MemoryAdmission(cause) => {
                CustomInvariantAccessDenialKind::MemoryAdmission(cause)
            }
        };
        Self::new(kind, subject)
    }

    pub fn new(kind: CustomInvariantAccessDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> CustomInvariantAccessDenialKind {
        self.kind
    }

    pub(super) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.subject.capacity() as u64
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for CustomInvariantAccessDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application invariant access denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for CustomInvariantAccessDenial {}
