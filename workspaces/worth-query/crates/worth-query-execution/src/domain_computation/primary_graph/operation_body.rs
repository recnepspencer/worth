//! Execution of a complete installed body, never a caller-selected replacement.
mod handler;
mod input;
mod installation;
mod source;
pub(in crate::domain_computation::primary_graph) use input::{prepare_inputs, PreparedBodyInputs};
pub(in crate::domain_computation::primary_graph) use source::PreparedBodySource;

pub(in crate::domain_computation::primary_graph) use handler::OperationBodyHandler;

use super::WorthQueryInvariantMutationTarget;
use serde::{Deserialize, Serialize};

/// A native body decision retains only its exact admitted scope mutation target.
pub struct WorthQueryOperationBodyDecision<Schema, Scope> {
    target: WorthQueryInvariantMutationTarget<Schema, Scope>,
    inputs: Option<PreparedBodyInputs>,
    sources: Vec<PreparedBodySource>,
}

/// A declared source expectation failed before any candidate effect.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorthQueryOperationBodyDenial {
    SourceMismatch {
        entity: String,
        aspect: String,
        field: String,
    },
    InputMismatch {
        member: String,
    },
}
