//! Execution of a complete installed body, never a caller-selected replacement.
mod handler;
mod installation;

pub(in crate::domain_computation::primary_graph) use handler::OperationBodyHandler;

use super::WorthQueryInvariantMutationTarget;
use serde::{Deserialize, Serialize};

/// A native body decision retains only its exact admitted scope mutation target.
pub struct WorthQueryOperationBodyDecision<Schema, Scope> {
    target: WorthQueryInvariantMutationTarget<Schema, Scope>,
}

/// A declared source expectation failed before any candidate effect.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorthQueryOperationBodyDenial {
    SourceMismatch {
        entity: String,
        aspect: String,
        field: String,
    },
}
