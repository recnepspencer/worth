//! Why a managed computation refused to go on: its owner stopped it, or its execution refused.

use {crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial, crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryManagedComputationInterruption, crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial};

#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryManagedComputationDenial<Stopped> {
    Owner(Stopped),
    Panicked,
    Resource(WorthQueryManagedComputationResourceDenial),
    Interrupted(WorthQueryManagedComputationInterruption),
    /// A pattern the computation ran stopped; its run reports why.
    NestedPatternStopped,
}

impl<Stopped: std::fmt::Debug> std::fmt::Display for WorthQueryManagedComputationDenial<Stopped> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "managed computation stopped: {self:?}")
    }
}

impl<Stopped: std::fmt::Debug + 'static> std::error::Error
    for WorthQueryManagedComputationDenial<Stopped>
{
}

impl<Stopped> From<WorthQueryManagedComputationCheckpointDenial>
    for WorthQueryManagedComputationDenial<Stopped>
{
    fn from(denial: WorthQueryManagedComputationCheckpointDenial) -> Self {
        match denial {
            WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
                Self::Resource(denial)
            }
            WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
                Self::Interrupted(interruption)
            }
            WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
                Self::NestedPatternStopped
            }
        }
    }
}
