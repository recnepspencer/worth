use serde::{Deserialize, Serialize};

/// Recovery possibility reported by C.8 without selecting or authorizing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalRecoveryOption {
    RetainCurrent,
    UsePrevious,
    ReconstructDerived,
    RequireOperator,
    NoneKnown,
}
