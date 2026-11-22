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

/// Domain outcome of one handler phase, keeping request interruption distinct.
#[derive(Debug)]
pub enum HandlerResult<Value, Denial> {
    Completed(Value),
    DomainDenied(Denial),
    ExecutionDenied(HandlerExecutionDenial),
    Cancelled,
    DeadlineExceeded,
}
