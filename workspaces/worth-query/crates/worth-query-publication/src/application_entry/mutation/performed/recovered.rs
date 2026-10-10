//! Successful one-use source promotion with independent recovery obligations.

use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationRequiredOutputRoot,
};
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryManagedApplicationRecoveryPerformed,
};
use worth_query_execution::facade::runtime::ProductUnpublishedCause;
use worth_query_installation::facade::ApplicationSchema;

use super::{RootConnection, WorthQueryRequiredOutputPreparation};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;

/// Output-source promotion is distinct from completing every independent
/// recovery cleanup. This result preserves the original cause and all owners.
pub struct WorthQueryRecoveredRequiredOutputs<Schema, Program, Root, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    pub(super) outputs: WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding>,
    pub(super) initial_cause: ProductUnpublishedCause,
    pub(super) performed: WorthQueryManagedApplicationRecoveryPerformed,
    pub(super) prior_cleanup: Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
}

impl<Schema, Program, Root, Binding>
    WorthQueryRecoveredRequiredOutputs<Schema, Program, Root, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    /// Moves the output continuation, initial diagnostic, full performed result
    /// and every earlier failed cleanup to the caller without dropping custody.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding>,
        ProductUnpublishedCause,
        WorthQueryManagedApplicationRecoveryPerformed,
        Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    ) {
        (
            self.outputs,
            self.initial_cause,
            self.performed,
            self.prior_cleanup,
        )
    }
}
