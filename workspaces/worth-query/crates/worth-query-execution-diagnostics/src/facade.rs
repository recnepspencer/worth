//! Explicit internal contract for Query execution diagnostic identity and
//! causal inspection protocols.

pub mod evidence_identity {
    pub use crate::evidence_identity::*;
}

pub mod causal_inspection {
    pub use crate::causal_inspection::*;
}

/// Internal materialization contract validator for Execution's court door.
/// Never re-exported on public execution facades.
pub mod materialization_contract {
    pub use crate::causal_inspection::materialization::contract::validate_materialization_contract;
}
