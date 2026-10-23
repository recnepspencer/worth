use std::sync::Arc;

use crate::validation::data::CustomInvariantSemanticIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomInvariantPreparationError {
    detail: Arc<str>,
}

impl CustomInvariantPreparationError {
    pub fn new(detail: impl Into<Arc<str>>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomInvariantExecutionError {
    detail: Arc<str>,
}

impl CustomInvariantExecutionError {
    pub fn new(detail: impl Into<Arc<str>>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomInvariantTraversalError {
    detail: Arc<str>,
}

impl CustomInvariantTraversalError {
    pub fn new(detail: impl Into<Arc<str>>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl From<CustomInvariantTraversalError> for CustomInvariantPreparationError {
    fn from(value: CustomInvariantTraversalError) -> Self {
        Self::new(value.detail)
    }
}

impl From<CustomInvariantTraversalError> for CustomInvariantExecutionError {
    fn from(value: CustomInvariantTraversalError) -> Self {
        Self::new(value.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomInvariantVerdict {
    Pass,
    Violation(super::CustomInvariantRefusal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomInvariantRuntimePhase {
    Preparation,
    Execution,
}

impl CustomInvariantRuntimePhase {
    pub(crate) const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::Preparation => "preparation",
            Self::Execution => "execution",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomInvariantFailureKind {
    PreparationError,
    ExecutionError,
    Panic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomInvariantFailure {
    pub(crate) identity: CustomInvariantSemanticIdentity,
    pub(crate) phase: CustomInvariantRuntimePhase,
    pub(crate) kind: CustomInvariantFailureKind,
    pub(crate) detail: Arc<str>,
}

impl CustomInvariantFailure {
    pub(crate) fn preparation_error(
        identity: &CustomInvariantSemanticIdentity,
        error: CustomInvariantPreparationError,
    ) -> Self {
        Self {
            identity: identity.clone(),
            phase: CustomInvariantRuntimePhase::Preparation,
            kind: CustomInvariantFailureKind::PreparationError,
            detail: Arc::from(error.detail()),
        }
    }

    pub(crate) fn execution_error(
        identity: &CustomInvariantSemanticIdentity,
        error: CustomInvariantExecutionError,
    ) -> Self {
        Self {
            identity: identity.clone(),
            phase: CustomInvariantRuntimePhase::Execution,
            kind: CustomInvariantFailureKind::ExecutionError,
            detail: Arc::from(error.detail()),
        }
    }

    pub(crate) fn panic(
        identity: &CustomInvariantSemanticIdentity,
        phase: CustomInvariantRuntimePhase,
        detail: Arc<str>,
    ) -> Self {
        Self {
            identity: identity.clone(),
            phase,
            kind: CustomInvariantFailureKind::Panic,
            detail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreparedCustomInvariantExecutionOutcome {
    Verdict(CustomInvariantVerdict),
    Failure(CustomInvariantFailure),
}
