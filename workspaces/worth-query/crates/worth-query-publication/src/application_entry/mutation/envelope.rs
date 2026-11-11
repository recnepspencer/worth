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
        ApplicationEditEnvelopeKey, ApplicationEditIntentIdentity, ApplicationEditStageIdentity,
        ApplicationEditStageIntent, ApplicationEncodedInput, ApplicationMutationAdmissionBinding,
        ApplicationMutationBinding, ApplicationMutationIdentityDenial, ApplicationMutationIntent,
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
        WorthQueryApplicationUncommitted, WorthQueryHistoricalApplicationCommit,
        WorthQueryMutationHandlerWork, WorthQueryPrimaryGraphApplicationRuntime,
    },
    runtime::ExecutionAllocationPolicy,
};
use worth_query_execution::publication_boundary::authored_edit::{
    stage_admission_refusal, AuthoredEditStageAdmission, AuthoredStageDomainDenial,
    AuthoredStageExecutionDenial, CapturedAuthoredEditStage, DeclaredEditDependency,
    DeclaredEditOrderRefusal, DeclaredEditScopeCycle, DeclaredEditStage,
    RetainedAuthoredStageResult,
};
use worth_query_installation::facade::ApplicationSchema;
mod builder;
mod execution_denial;
pub use execution_denial::{WorthQueryEditEnvelopeExecutionDenial, WorthQueryEditEnvelopeSubject};
mod entry;
mod outcome;
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
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
    principal: &'a worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<S>,
    scope: &'a worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
}
pub(super) type StageValue = Arc<Mutex<Option<Box<dyn Any + Send>>>>;
pub(super) type Connection<S, R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
pub(super) type Demand<S, R> =
    <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
pub(super) type Family<S, R> = <Demand<S, R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
pub(super) type Source<S, R> = <Family<S, R> as WorthQueryProducerOutputFamily<S>>::Source;
