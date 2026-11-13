//! Authored stages share the ordinary mutation protocol and one Native candidate.
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, HandlerResult, MutationHandlerExecutionDenial,
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use std::any::Any;
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::{
    ApplicationEditStageIdentity, ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_program::WorthQueryApplicationOutputDemand;
use worth_query_installation::facade::ApplicationSchema;
mod capture;
mod dag;
mod dependencies;
mod envelope;
mod envelope_admission;
mod execution;
mod planning;
pub use dependencies::DeclaredQueryRead;
pub(in crate::domain_computation::primary_graph::application_contribution) use dependencies::{
    managed_reads, ProducerDependencyScopes,
};
pub use envelope::{
    with_resolved_edit_envelope, AuthoredEnvelopeResolution, UnseenAuthoredEnvelope,
};
mod results;
pub(in crate::domain_computation::primary_graph) use capture::AuthoredEditStageExecutor;
pub use capture::{AuthoredEditStageAdmission, CapturedAuthoredEditStage};
pub use dag::{
    order_authored_stages, DeclaredEditDependency, DeclaredEditOrderRefusal,
    DeclaredEditParticipant, DeclaredEditReadScope, DeclaredEditScopeCycle, DeclaredEditStage,
};
pub use results::{
    AuthoredStageDomainDenial, AuthoredStageExecutionCause, AuthoredStageExecutionDenial,
    RetainedAuthoredStageResult,
};

pub type AuthoredEditStageOperation<S, B> =
    WorthQueryAdmittedApplicationOperation<
        S,
        <B as ApplicationMutationBinding<S>>::Operation,
        <B as ApplicationMutationBinding<S>>::Input,
        <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<
            S,
        >>::Scope,
    >;

/// Publication supplies genuine C authorization. This carrier grants nothing
/// beyond the admitted operation it owns.
pub struct AuthoredEditStageContext<'a, 'run, 'phase, S> {
    pub(in crate::domain_computation::primary_graph) phase:
        &'run WorthQueryAdvancementPhase<'phase>,
    pub(in crate::domain_computation::primary_graph) application:
        &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
}
impl<S> AuthoredEditStageContext<'_, '_, '_, S> {
    pub fn phase(&self) -> &WorthQueryAdvancementPhase<'_> {
        self.phase
    }
    pub fn application(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<S> {
        self.application
    }
}

/// Publication reports a failed genuine admission with its nominal stage subject.
pub fn stage_admission_refusal<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ordinal: usize,
    detail: String,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::AuthoredStageRefused {
            ordinal,
            binding: B::IDENTITY,
        },
        detail,
    )
}
