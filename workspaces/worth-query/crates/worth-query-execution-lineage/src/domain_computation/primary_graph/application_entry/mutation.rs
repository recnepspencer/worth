pub use super::super::handler::HandlerInterruption;
pub mod decision_plan;
pub mod durable_capability;
pub use decision_plan::WorthQueryDecisionPlan;
pub use durable_capability::WorthQueryOperationDurableCapability;
pub mod home_retirement;
pub use home_retirement::{
    WorthQueryOperationHomeRetirementDenial, WorthQueryOperationHomeRetirementProgress,
};
pub mod execution;
pub mod execution_report;
pub mod failure;
mod keyed_entity;
pub use execution::MutationHandlerExecutionDenial;
pub use execution_report::{
    WorthQueryMutationHandlerExecutionReport, WorthQueryMutationHandlerProjectionWork,
    WorthQueryMutationHandlerWork,
};
pub use failure::HandlerFailure;

/// Execution failure reported by handler-owned decision or candidate work.
#[derive(Debug)]
pub struct HandlerExecutionDenial {
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl HandlerExecutionDenial {
    /// The exact Native stop retained by ordinary or managed-index decision preparation.
    pub fn native_decision_refusal(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::CompanionPreflightStop> {
        execution::native_decision_cause(self.source.as_ref()).or_else(|| {
            crate::domain_computation::primary_graph::invariant_projection::native_index_decision_cause(self.source.as_ref())
        })
    }

    pub fn new(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }

    /// Recovers an owning product's concrete handler denial without exposing
    /// that product type to Query.
    pub fn downcast<Denial>(self) -> Result<Denial, Self>
    where
        Denial: std::error::Error + Send + Sync + 'static,
    {
        match self.source.downcast::<Denial>() {
            Ok(denial) => Ok(*denial),
            Err(source) => Err(Self { source }),
        }
    }
}

impl std::fmt::Display for HandlerExecutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(formatter)
    }
}

impl std::error::Error for HandlerExecutionDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl From<super::super::application_attempt::WorthQueryApplicationAttemptDenial>
    for HandlerExecutionDenial
{
    fn from(source: super::super::application_attempt::WorthQueryApplicationAttemptDenial) -> Self {
        Self::new(source)
    }
}

impl From<super::super::invariant_projection::WorthQueryInvariantProjectionTraversalDenial>
    for HandlerExecutionDenial
{
    fn from(
        source: super::super::invariant_projection::WorthQueryInvariantProjectionTraversalDenial,
    ) -> Self {
        Self::new(source)
    }
}

/// Domain outcome of one handler phase, keeping request interruption distinct.
#[derive(Debug)]
pub enum HandlerResult<Value, Denial> {
    Completed(Value),
    DomainDenied(Denial),
    ExecutionDenied(HandlerExecutionDenial),
    Cancelled,
    DeadlineExceeded,
}

impl<Value, Denial> HandlerResult<Value, Denial>
where
    Denial: From<super::super::WorthQueryDomainRefusal>,
{
    /// Refuses with the application's bounded domain reason, for example
    /// `HandlerResult::domain_refusal("lot_expired", "Lot 7 expired on 3 May.")`.
    ///
    /// An inadmissible code or message is a handler defect, not a business
    /// refusal: it returns `ExecutionDenied` carrying the
    /// `WorthQueryDomainRefusalDenial`, never a truncated reason or a panic.
    pub fn domain_refusal(code: &str, message: &str) -> Self {
        match super::super::WorthQueryDomainRefusal::new(code, message) {
            Ok(refusal) => Self::DomainDenied(refusal.into()),
            Err(denial) => Self::ExecutionDenied(HandlerExecutionDenial::new(denial)),
        }
    }
}

impl<Value, Denial, Execution> From<Result<Value, Execution>> for HandlerResult<Value, Denial>
where
    Execution: Into<HandlerExecutionDenial>,
{
    fn from(result: Result<Value, Execution>) -> Self {
        match result {
            Ok(value) => Self::Completed(value),
            Err(denial) => Self::ExecutionDenied(denial.into()),
        }
    }
}

#[cfg(test)]
mod handler_denial_conversion_tests {
    use super::*;
    use crate::domain_computation::primary_graph::{
        application_attempt::{
            WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
        },
        invariant_projection::{
            WorthQueryInvariantProjectionTraversalDenial,
            WorthQueryInvariantProjectionTraversalDenialKind,
        },
    };

    #[test]
    fn attempt_and_traversal_denials_convert_into_handler_execution_denial() {
        let attempt = WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
            "field-write",
        );
        let execution: HandlerExecutionDenial = attempt.into();
        let recovered = execution
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .expect("original attempt denial stays recoverable");
        assert_eq!(
            recovered.kind(),
            WorthQueryApplicationAttemptDenialKind::UndeclaredEffect
        );
        assert_eq!(recovered.subject(), "field-write");

        let traversal = WorthQueryInvariantProjectionTraversalDenial::new(
            WorthQueryInvariantProjectionTraversalDenialKind::RelationNotInstalled,
            "links",
        );
        let result: HandlerResult<u32, ()> = Err(traversal).into();
        let HandlerResult::ExecutionDenied(execution) = result else {
            panic!("traversal error converts into ExecutionDenied");
        };
        let recovered = execution
            .downcast::<WorthQueryInvariantProjectionTraversalDenial>()
            .expect("original traversal denial stays recoverable");
        assert_eq!(recovered.relation(), "links");

        let ok: HandlerResult<u32, ()> = Ok::<u32, HandlerExecutionDenial>(42).into();
        assert!(matches!(ok, HandlerResult::Completed(42)));
    }
}
