//! Move-only custody for a discovered source whose World effects need recovery.

use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_execution::facade::application_installation::WorthQueryUnpublishedProgramOutputSource;
use worth_query_execution::facade::domain_computation::{
    WorthQueryProductUnpublishedApplication, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryManagedApplicationRecoveryPerformed,
};
use worth_query_execution::facade::runtime::ProductUnpublishedCause;
use worth_query_installation::facade::ApplicationSchema;

use super::{Discovery, RootConnection};

pub(super) use crate::application_entry::mutation::performed_source::recovery::{
    ProgramSourceRecoveryPhase as DiscoveredRecoveryPhase, RecoveredCarrier,
};

/// Original discovered-root preparation and exact native partial. Recovery
/// never reruns the handler. Promotion consumes this move-only owner.
pub struct WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub(super) source: WorthQueryUnpublishedProgramOutputSource,
    pub(super) discovery: Discovery<Schema, Root>,
    pub(super) phase: DiscoveredRecoveryPhase,
    pub(super) prior_cleanup: Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) initial_cause: ProductUnpublishedCause,
    pub(super) program: std::marker::PhantomData<fn() -> (Schema, Intent, Program, Root)>,
}

impl<Schema, Intent, Program, Root>
    WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub(super) fn new(
        source: WorthQueryUnpublishedProgramOutputSource,
        discovery: Discovery<Schema, Root>,
        partial: WorthQueryProductUnpublishedApplication,
    ) -> Self {
        Self {
            initial_cause: partial.cause(),
            source,
            discovery,
            phase: DiscoveredRecoveryPhase::Unpublished(partial.into_recovery()),
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
            DiscoveredRecoveryPhase::Performed { outcome, .. } => Some(outcome),
            DiscoveredRecoveryPhase::Unpublished(_) => None,
        }
    }

    pub fn prior_cleanup_failures(&self) -> &[WorthQueryProductUnpublishedRecoveryReleaseFailure] {
        &self.prior_cleanup
    }
}

/// Native program-source progress, shared with fixed required roots.
pub type WorthQueryDiscoveredRecoveryProgress = crate::application_entry::mutation::performed_source::recovery::WorthQueryProgramSourceRecoveryProgress;
