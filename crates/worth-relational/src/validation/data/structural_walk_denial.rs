use super::structural_read::StructuralReadError;
use crate::validation::custom_rule::interruption::CustomInvariantInterruption;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralWalkDenialKind {
    Interrupted(CustomInvariantInterruption),
    Read(StructuralReadError),
    MemoryBudgetExceeded,
    MemoryAdmission(worth_execution::MemoryLimitDenial),
}
