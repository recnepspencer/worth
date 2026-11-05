//! Sealed test doors over this module's private members.
use super::*;

pub trait RepublicationDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    #[doc(hidden)]
    fn record_restored_generated_output_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        producer: ProducerQualification,
    ) {
        Self::record_restored_generated_output(
            self,
            execution,
            observation,
            correspondence,
            producer,
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > RepublicationDoorsOps<Schema> for HostRuntime
{
}
