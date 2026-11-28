use std::sync::Arc;

use crate::validation::data::rule_id::CustomInvariantSemanticIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomInvariantPreparationError {
    Runtime(Arc<str>),
    Cause(Box<crate::validation::data::custom_rule::failure_cause::CustomInvariantFailureCause>),
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
    fn into_parts(
        self,
    ) -> (
        Arc<str>,
        Option<
            Box<crate::validation::data::custom_rule::failure_cause::CustomInvariantFailureCause>,
        >,
    ) {
        match self {
            Self::Cause(cause) => (Arc::from("typed invariant failure"), Some(cause)),
            Self::Runtime(detail) => (detail, None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomInvariantExecutionError {
    Runtime(Arc<str>),
    Cause(Box<crate::validation::data::custom_rule::failure_cause::CustomInvariantFailureCause>),
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
    fn into_parts(
        self,
    ) -> (
        Arc<str>,
        Option<
            Box<crate::validation::data::custom_rule::failure_cause::CustomInvariantFailureCause>,
        >,
    ) {
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
    Violation(crate::validation::data::custom_rule::refusal::CustomInvariantRefusal),
}

pub use crate::validation::data::results::{
    CustomInvariantFailureKind, CustomInvariantFailurePhase,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomInvariantFailure {
    pub identity: CustomInvariantSemanticIdentity,
    pub phase: CustomInvariantFailurePhase,
    pub kind: CustomInvariantFailureKind,
    pub detail: Arc<str>,
    pub cause: Option<
        Box<crate::validation::data::custom_rule::failure_cause::CustomInvariantFailureCause>,
    >,
}

impl CustomInvariantFailure {
    pub fn preparation_error(
        identity: &CustomInvariantSemanticIdentity,
        error: CustomInvariantPreparationError,
    ) -> Self {
        let (detail, cause) = error.into_parts();
        Self {
            identity: identity.clone(),
            phase: CustomInvariantFailurePhase::Preparation,
            kind: CustomInvariantFailureKind::PreparationError,
            detail,
            cause,
        }
    }

    pub fn execution_error(
        identity: &CustomInvariantSemanticIdentity,
        error: CustomInvariantExecutionError,
    ) -> Self {
        let (detail, cause) = error.into_parts();
        Self {
            identity: identity.clone(),
            phase: CustomInvariantFailurePhase::Execution,
            kind: CustomInvariantFailureKind::ExecutionError,
            detail,
            cause,
        }
    }

    pub fn panic(
        identity: &CustomInvariantSemanticIdentity,
        phase: CustomInvariantFailurePhase,
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
pub enum PreparedCustomInvariantExecutionOutcome {
    Verdict(CustomInvariantVerdict),
    Failure(CustomInvariantFailure),
}
