//! Original committed discovered source awaiting registration, never receipt reconstruction.
use super::*;
/// Move-only original result, discovery and native unregistered carrier.
/// Borrowing its diagnostics grants no execution or retention authority.
/// ```compile_fail
/// use worth_query_publication::facade::application_entry::WorthQueryDiscoveredOutputRetentionFailure as Failure;
/// use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
/// use worth_query_declaration::facade::application_program::{ApplicationOutputGraphShape, ApplicationDiscoveredOutputRoot, ApplicationProgramDefinition, ApplicationConnectionShape};
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
/// fn cannot_clone<S,I,P,R>(failure: Failure<S,I,P,R>)
/// where S: ApplicationSchema, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
/// R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationDiscoveredOutputConnection<S, Source=I::Binding>,
/// { let _copy = failure.clone(); }
/// ```
/// ```compile_fail,E0451
/// use worth_query_publication::facade::application_entry::WorthQueryDiscoveredOutputRetentionFailure as Failure;
/// use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
/// use worth_query_declaration::facade::application_program::{ApplicationOutputGraphShape, ApplicationDiscoveredOutputRoot, ApplicationProgramDefinition, ApplicationConnectionShape};
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
/// fn cannot_reconstruct<S,I,P,R>(failure: Failure<S,I,P,R>)
/// where S: ApplicationSchema, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
/// R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationDiscoveredOutputConnection<S, Source=I::Binding>,
/// { let _forged = Failure { ..failure }; }
/// ```
/// Borrowed evidence remains available without transferring custody.
/// ```
/// use worth_query_publication::facade::application_entry::WorthQueryDiscoveredOutputRetentionFailure as Failure;
/// use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
/// use worth_query_declaration::facade::application_program::{ApplicationOutputGraphShape, ApplicationDiscoveredOutputRoot, ApplicationProgramDefinition, ApplicationConnectionShape};
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
/// fn borrow<S,I,P,R>(failure: &Failure<S,I,P,R>)
/// where S: ApplicationSchema, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
/// R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationDiscoveredOutputConnection<S, Source=I::Binding>,
/// { let _ = (failure.receipt(), failure.result(), failure.denial()); }
/// ```
#[must_use = "retain original committed custody for checked retry or explicit abandonment"]
pub struct WorthQueryDiscoveredOutputRetentionFailure<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
    pub(super) result: MutationResult<Schema, Intent>,
    pub(super) denial: WorthQueryRequiredOutputPreparationDenial,
    pub(super) custody: Option<worth_query_execution::facade::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure>,
    pub(super) discovery: Discovery<Schema, Root>,
    pub(super) program: std::marker::PhantomData<fn() -> Program>,
}
impl<S, I, P, R> WorthQueryDiscoveredOutputRetentionFailure<S, I, P, R>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
{
    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
    pub fn result(&self) -> &MutationResult<S, I> {
        &self.result
    }
    pub fn denial(&self) -> &WorthQueryRequiredOutputPreparationDenial {
        &self.denial
    }
    /// Actual native carrier availability and posture, not a permission to retry without admission.
    pub fn recovery_posture(
        &self,
    ) -> crate::application_entry::WorthQueryRequiredOutputRecoveryPosture {
        use crate::application_entry::WorthQueryRequiredOutputRecoveryPosture as Posture;
        match self.custody.as_ref() {
            Some(custody) if custody.has_original_delivery() => {
                WorthQueryRequiredOutputPreparationDenial::DemandExecution(custody.denial().clone())
                    .recovery_posture()
            }
            _ => Posture::Terminal,
        }
    }
}
