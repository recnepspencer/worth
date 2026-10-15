//! Move-only custody for a required source whose World effects need recovery.

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
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
pub struct WorthQueryUnpublishedRequiredApplicationMutation<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding:
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
    pub(super) program: std::marker::PhantomData<fn() -> (Schema, Binding, Program, Root)>,
}

impl<Schema, Binding, Program, Root>
    WorthQueryUnpublishedRequiredApplicationMutation<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding:
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

    pub(in crate::application_entry) fn performed_mut(
        &mut self,
    ) -> Option<&mut WorthQueryManagedApplicationRecoveryPerformed> {
        match &mut self.phase {
            ProgramSourceRecoveryPhase::Performed { outcome, .. } => Some(outcome),
            _ => None,
        }
    }
    pub(in crate::application_entry) fn take_prior_cleanup(
        &mut self,
    ) -> Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure> {
        std::mem::take(&mut self.prior_cleanup)
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

    pub fn recovery_posture(&self) -> super::WorthQueryRequiredOutputRecoveryPosture {
        match &self.phase {
            ProgramSourceRecoveryPhase::Performed {
                carrier: RecoveredCarrier::Unavailable,
                ..
            } => super::WorthQueryRequiredOutputRecoveryPosture::Terminal,
            ProgramSourceRecoveryPhase::Performed {
                carrier: RecoveredCarrier::Retained(carrier),
                ..
            } if !carrier.has_original_delivery() => {
                super::WorthQueryRequiredOutputRecoveryPosture::Terminal
            }
            ProgramSourceRecoveryPhase::Unpublished(_)
            | ProgramSourceRecoveryPhase::Performed {
                carrier: RecoveredCarrier::Retained(_),
                ..
            } => super::WorthQueryRequiredOutputRecoveryPosture::Retryable,
        }
    }

    pub fn prior_cleanup_failures(&self) -> &[WorthQueryProductUnpublishedRecoveryReleaseFailure] {
        &self.prior_cleanup
    }
}
