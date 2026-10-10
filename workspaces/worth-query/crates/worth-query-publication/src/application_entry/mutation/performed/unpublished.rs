//! Move-only custody for a required source whose World effects need recovery.

use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationRequiredOutputRoot,
};
use worth_query_execution::facade::application_installation::WorthQueryUnpublishedProgramOutputSource;
use worth_query_execution::facade::domain_computation::{
    WorthQueryProductUnpublishedApplication, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryManagedApplicationRecoveryPerformed,
};
use worth_query_execution::facade::runtime::ProductUnpublishedCause;
use worth_query_installation::facade::ApplicationSchema;

use super::{ProgramDemand, RootConnection};

pub(super) use crate::application_entry::mutation::performed_source::recovery::{
    ProgramSourceRecoveryPhase, RecoveredCarrier,
};

/// Original required-root preparation and exact native partial. Recovery
/// never reruns the handler. Promotion consumes this move-only owner.
pub struct WorthQueryUnpublishedRequiredApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputSource<
            Schema,
            RootConnection<Schema, Root>,
        >,
{
    pub(super) source: WorthQueryUnpublishedProgramOutputSource,
    pub(super) demand: ProgramDemand<Schema, Root>,
    pub(super) phase: ProgramSourceRecoveryPhase,
    pub(super) prior_cleanup: Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) initial_cause: ProductUnpublishedCause,
    pub(super) program: std::marker::PhantomData<fn() -> (Schema, Intent, Program, Root)>,
}

impl<Schema, Intent, Program, Root>
    WorthQueryUnpublishedRequiredApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputSource<
            Schema,
            RootConnection<Schema, Root>,
        >,
{
    pub(super) fn new(
        source: WorthQueryUnpublishedProgramOutputSource,
        demand: ProgramDemand<Schema, Root>,
        partial: WorthQueryProductUnpublishedApplication,
    ) -> Self {
        Self {
            initial_cause: partial.cause(),
            source,
            demand,
            phase: ProgramSourceRecoveryPhase::Unpublished(partial.into_recovery()),
            prior_cleanup: Vec::new(),
            program: std::marker::PhantomData,
        }
    }

    pub fn initial_cause(&self) -> ProductUnpublishedCause {
        self.initial_cause
    }

    /// The raw performed result, including failed read/publication/cleanup.
    /// Its presence alone does not certify output-source promotion.
    pub fn performed(&self) -> Option<&WorthQueryManagedApplicationRecoveryPerformed> {
        match &self.phase {
            ProgramSourceRecoveryPhase::Performed { outcome, .. } => Some(outcome),
            ProgramSourceRecoveryPhase::Unpublished(_) => None,
        }
    }

    pub fn prior_cleanup_failures(&self) -> &[WorthQueryProductUnpublishedRecoveryReleaseFailure] {
        &self.prior_cleanup
    }
}
