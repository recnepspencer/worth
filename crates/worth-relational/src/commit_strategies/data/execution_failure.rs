use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyExecutorFailureClass {
    InvalidInput,
    ReadContractViolation,
    ProjectionContractViolation,
    DomainRejection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyExecutorFailureEvidence {
    AspectFieldLocator {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        locator: worth_foundational::facade::AspectFieldLocator,
    },
    AspectFieldLocatorMismatch {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        locator: worth_foundational::facade::AspectFieldLocator,
        expected_aspect_key: worth_foundational::facade::AspectKey,
    },
}

/// A transient executor refusal: request control cannot become a serialized domain failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyExecutorFailure {
    Request(crate::execution::RelationalExecutionDenialCause),
    Domain {
        class: StrategyExecutorFailureClass,
        detail: Arc<str>,
        evidence: Option<StrategyExecutorFailureEvidence>,
    },
}

impl StrategyExecutorFailure {
    pub fn new(class: StrategyExecutorFailureClass, detail: impl Into<Arc<str>>) -> Self {
        Self::Domain {
            class,
            detail: detail.into(),
            evidence: None,
        }
    }

    pub fn with_evidence(
        class: StrategyExecutorFailureClass,
        detail: impl Into<Arc<str>>,
        evidence: StrategyExecutorFailureEvidence,
    ) -> Self {
        Self::Domain {
            class,
            detail: detail.into(),
            evidence: Some(evidence),
        }
    }
}
