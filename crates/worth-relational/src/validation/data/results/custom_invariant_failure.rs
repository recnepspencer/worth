use serde::{Deserialize, Serialize};

use crate::validation::data::rule_id::CustomInvariantSemanticIdentity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomInvariantFailureIdentity {
    semantic_identity: CustomInvariantSemanticIdentity,
}

impl CustomInvariantFailureIdentity {
    pub fn new(semantic_identity: CustomInvariantSemanticIdentity) -> Self {
        Self { semantic_identity }
    }

    pub fn semantic_identity(&self) -> &CustomInvariantSemanticIdentity {
        &self.semantic_identity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomInvariantFailurePhase {
    Preparation,
    Execution,
}

impl CustomInvariantFailurePhase {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::Preparation => "preparation",
            Self::Execution => "execution",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomInvariantFailureKind {
    PreparationError,
    ExecutionError,
    Panic,
}

impl CustomInvariantFailureKind {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::PreparationError => "preparation_error",
            Self::ExecutionError => "execution_error",
            Self::Panic => "panic",
        }
    }
}
