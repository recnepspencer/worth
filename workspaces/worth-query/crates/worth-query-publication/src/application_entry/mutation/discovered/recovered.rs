//! Successful one-use source promotion with independent recovery obligations.

use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryManagedApplicationRecoveryPerformed,
};
use worth_query_execution::facade::runtime::ProductUnpublishedCause;
use worth_query_installation::facade::ApplicationSchema;

use super::{RootConnection, WorthQueryDiscoveredProgramOutputHandle};

/// Output-source promotion is distinct from completing every independent
/// recovery cleanup. This result preserves the original cause and all owners.
pub struct WorthQueryRecoveredDiscoveredOutputs<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    pub(super) outputs: WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
    pub(super) initial_cause: ProductUnpublishedCause,
    pub(super) performed: WorthQueryManagedApplicationRecoveryPerformed,
    pub(super) prior_cleanup: Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
}

impl<Schema, Program, Root> WorthQueryRecoveredDiscoveredOutputs<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    /// Moves the output continuation, initial diagnostic, full performed result
    /// and every earlier failed cleanup to the caller without dropping custody.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
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
