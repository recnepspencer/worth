use std::sync::Arc;

use crate::validation::data::CustomInvariantSemanticIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomInvariantPreparationError {
    Runtime(Arc<str>),
    Cause(Box<super::CustomInvariantFailureCause>),
}
impl CustomInvariantPreparationError {
    pub fn new(detail: impl Into<Arc<str>>) -> Self {
        Self::Runtime(detail.into())
    }
    pub fn detail(&self) -> &str {
        match self {
            Self::Runtime(detail) => detail,
            Self::Cause(_) => "typed invariant failure",
        }
    }
    fn into_parts(self) -> (Arc<str>, Option<Box<super::CustomInvariantFailureCause>>) {
        match self {
            Self::Cause(cause) => (Arc::from("typed invariant failure"), Some(cause)),
            Self::Runtime(detail) => (detail, None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomInvariantExecutionError {
    Runtime(Arc<str>),
    Cause(Box<super::CustomInvariantFailureCause>),
}
impl CustomInvariantExecutionError {
    pub fn new(detail: impl Into<Arc<str>>) -> Self {
        Self::Runtime(detail.into())
    }
    pub fn detail(&self) -> &str {
        match self {
            Self::Runtime(detail) => detail,
            Self::Cause(_) => "typed invariant failure",
        }
    }
    fn into_parts(self) -> (Arc<str>, Option<Box<super::CustomInvariantFailureCause>>) {
        match self {
            Self::Cause(cause) => (Arc::from("typed invariant failure"), Some(cause)),
            Self::Runtime(detail) => (detail, None),
        }
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
    pub(crate) cause: Option<Box<super::CustomInvariantFailureCause>>,
}

impl CustomInvariantFailure {
    pub(crate) fn preparation_error(
        identity: &CustomInvariantSemanticIdentity,
        error: CustomInvariantPreparationError,
    ) -> Self {
        let (detail, cause) = error.into_parts();
        Self {
            identity: identity.clone(),
            phase: CustomInvariantRuntimePhase::Preparation,
            kind: CustomInvariantFailureKind::PreparationError,
            detail,
            cause,
        }
    }

    pub(crate) fn execution_error(
        identity: &CustomInvariantSemanticIdentity,
        error: CustomInvariantExecutionError,
    ) -> Self {
        let (detail, cause) = error.into_parts();
        Self {
            identity: identity.clone(),
            phase: CustomInvariantRuntimePhase::Execution,
            kind: CustomInvariantFailureKind::ExecutionError,
            detail,
            cause,
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
            cause: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreparedCustomInvariantExecutionOutcome {
    Verdict(CustomInvariantVerdict),
    Failure(CustomInvariantFailure),
}
