//! A request-local authored edit with one key and one publication.
use crate::application_entry::{
    WorthQueryApplicationRequest, WorthQueryApplicationRequestMutationDenial,
};
use std::{
    any::Any,
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationEditEnvelopeKey, ApplicationEditEnvelopeOrder, ApplicationEditIntentIdentity,
        ApplicationEditStageIdentity, ApplicationEditStageIntent, ApplicationEncodedInput,
        ApplicationMutationAdmissionBinding, ApplicationMutationBinding,
        ApplicationMutationIdentityDenial, ApplicationMutationIntent,
        ApplicationMutationScopeResolution, NoApplicationMutationSource,
    },
    application_program::{
        ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
        ApplicationRequiredOutputRoot, WorthQueryApplicationOutputDemand,
        WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
        WorthQueryProducerOutputFamily,
    },
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementPhase,
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::{
        HandlerExecutionDenial, MutationHandlerExecutionDenial, WorthQueryApplicationCommitReceipt,
        WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationProjection,
        WorthQueryApplicationUncommitted, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryRecoveredApplicationCommitReceipt,
    },
    runtime::ExecutionAllocationPolicy,
};
use worth_query_execution::publication_boundary::authored_edit::{
    stage_admission_refusal, with_resolved_edit_envelope, AuthoredEditStageAdmission,
    AuthoredEnvelopeResolution, AuthoredStageDomainDenial, AuthoredStageExecutionDenial,
    CapturedAuthoredEditStage, DeclaredEditDependency, DeclaredEditOrderRefusal,
    DeclaredEditScopeCycle, DeclaredEditStage, RetainedAuthoredStageResult, UnseenAuthoredEnvelope,
};
use worth_query_installation::facade::ApplicationSchema;
mod builder;
pub(in crate::application_entry) mod runtime;
pub use runtime::{WorthQueryEditEnvelopeModeDenial, WorthQueryRuntimeEditEnvelope};
mod execution_denial;
pub use execution_denial::{WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeSubject};
pub(in crate::application_entry) mod entry;
mod outcome;
mod replay;
pub use builder::{
    NoEditStages, WithEditStage, WorthQueryEditEnvelope, WorthQueryKeyedEditEnvelope,
};
pub use entry::EditStageList;
pub use outcome::{
    WorthQueryEditEnvelopeOutcome, WorthQueryEditEnvelopeReceipt, WorthQueryEditStageDomainDenial,
    WorthQueryEditStageExecutionDenial, WorthQueryEditStageResultDenial, WorthQueryEditStageToken,
};
pub use worth_query_execution::publication_boundary::authored_edit::AuthoredStageExecutionCause;

#[doc(hidden)]
pub struct EnvelopeContext<'a, S> {
    pub(in crate::application_entry) application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    pub(in crate::application_entry) principal: &'a worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<S>,
    pub(in crate::application_entry) scope: &'a worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub(in crate::application_entry) branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    pub(in crate::application_entry) order: ApplicationEditEnvelopeOrder,
    pub(in crate::application_entry) correction: Option<worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionIntent>,
}
pub(super) type StageValue = Arc<Mutex<Option<Box<dyn Any + Send>>>>;
pub(in crate::application_entry) type Connection<S, R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
pub(in crate::application_entry) type Demand<S, R> =
    <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
pub(super) type Family<S, R> = <Demand<S, R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
pub(in crate::application_entry) type Source<S, R> =
    <Family<S, R> as WorthQueryProducerOutputFamily<S>>::Source;
