//! Domain denial and verdict vocabulary for installed application invariants.

pub use worth_relational::facade::runtime::CustomInvariantInterruption;

pub mod access_denial;
pub mod hook_error;
pub mod refusal;

pub use super::aggregate_identity::WorthQueryAggregateIdentity;
pub use access_denial::{WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind};
pub use hook_error::WorthQueryApplicationInvariantError;
pub use refusal::{WorthQueryEmptyInvariantRefusal, WorthQueryInvariantRefusalDecodeDenial};

/// A domain violation is distinct from failure to evaluate and interruption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationInvariantVerdict<Refusal> {
    Pass,
    Violation(Refusal),
}
