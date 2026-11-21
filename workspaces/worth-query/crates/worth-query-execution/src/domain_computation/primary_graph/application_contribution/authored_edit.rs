//! Thin outer-runtime wrappers for authored edit stages.

use std::any::Any;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::application_operation::{
    ApplicationEditEnvelopeKey, ApplicationEditIntentIdentity, ApplicationEncodedInput,
    ApplicationMutationBinding,
};
use worth_query_declaration::facade::application_program::WorthQueryApplicationOutputDemand;
use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::authored_edit as inner;
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    HandlerResult, WorthQueryAdvancementPhase, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryCompletedMutationCandidate, WorthQueryCorrectionIntent,
    WorthQueryMutationHandlerWork, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub use inner::{
    stage_admission_refusal, AuthoredEditStageAdmission, AuthoredEnvelopeResolution,
    AuthoredEnvelopeResolutionPurpose, AuthoredStageDomainDenial, AuthoredStageExecutionCause,
    AuthoredStageExecutionDenial, DeclaredEditDependency, DeclaredEditOrderRefusal,
    DeclaredEditParticipant, DeclaredEditReadScope, DeclaredEditScopeCycle, DeclaredEditStage,
    DeclaredQueryRead, RetainedAuthoredStageResult, UnseenAuthoredEnvelope,
};

pub struct AuthoredEditStageContext<'a, 'run, 'phase, S> {
    phase: &'run WorthQueryAdvancementPhase<'phase>,
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
}

impl<'a, S> AuthoredEditStageContext<'a, '_, '_, S> {
    pub fn phase(&self) -> &WorthQueryAdvancementPhase<'_> {
        self.phase
    }

    pub fn application(&self) -> &'a WorthQueryPrimaryGraphApplicationRuntime<S> {
        self.application
    }
}

pub struct CapturedAuthoredEditStage<S>(pub(crate) inner::CapturedAuthoredEditStage<S>);

impl<S> Clone for CapturedAuthoredEditStage<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: 'static> From<CapturedAuthoredEditStage<S>>
    for worth_query_execution_attempt::facade::ports::OpaqueAuthoredEditStage<S>
{
    fn from(stage: CapturedAuthoredEditStage<S>) -> Self {
        stage.0.into()
    }
}

fn adapt_admit<S, B>(
    admit: impl Fn(
            &AuthoredEditStageContext<'_, '_, '_, S>,
        ) -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial>
        + Send
        + Sync
        + 'static,
) -> impl Fn(
    &inner::AuthoredEditStageContext<'_, '_, '_, S>,
) -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial>
       + Send
       + Sync
       + 'static
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    move |context| {
        let application = context
            .outer_application()
            .and_then(|any| any.downcast_ref::<WorthQueryPrimaryGraphApplicationRuntime<S>>())
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    "missing outer application runtime for authored edit stage",
                )
            })?;
        admit(&AuthoredEditStageContext {
            phase: context.phase(),
            application,
        })
    }
}

impl<S: ApplicationSchema> CapturedAuthoredEditStage<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn capture<B>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        key: Arc<ApplicationEditEnvelopeKey<B::IdempotencyKey>>,
        input: ApplicationEncodedInput<B::InputBinding>,
        ordinal: usize,
        principal: WorthQueryAuthenticatedExternalPrincipal<S>,
        admit: impl Fn(
                &AuthoredEditStageContext<'_, '_, '_, S>,
            )
                -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial>
            + Send
            + Sync
            + 'static,
        result: Arc<std::sync::Mutex<Option<Box<dyn Any + Send>>>>,
    ) -> Result<Self, WorthQueryOutputDemandDenial>
    where
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
        B::IdempotencyKey: Send + Sync,
        B::Result: Send,
        B::Denial: Send,
    {
        inner::CapturedAuthoredEditStage::capture::<B>(
            access,
            phase,
            application,
            key,
            input,
            ordinal,
            principal,
            adapt_admit(admit),
            result,
        )
        .map(Self)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn capture_admitted<B>(
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        key: Arc<ApplicationEditEnvelopeKey<B::IdempotencyKey>>,
        input: ApplicationEncodedInput<B::InputBinding>,
        ordinal: usize,
        principal: WorthQueryAuthenticatedExternalPrincipal<S>,
        admitted: AuthoredEditStageAdmission<S, B>,
        admit: impl Fn(
                &AuthoredEditStageContext<'_, '_, '_, S>,
            )
                -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial>
            + Send
            + Sync
            + 'static,
        result: Arc<std::sync::Mutex<Option<Box<dyn Any + Send>>>>,
    ) -> Result<Self, WorthQueryOutputDemandDenial>
    where
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
        B::IdempotencyKey: Send + Sync,
        B::Result: Send,
        B::Denial: Send,
    {
        inner::CapturedAuthoredEditStage::capture_admitted::<B>(
            access,
            application,
            key,
            input,
            ordinal,
            principal,
            admitted,
            adapt_admit(admit),
            result,
        )
        .map(Self)
    }

    pub fn prepare_prefix_candidate<B: ApplicationMutationBinding<S>>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        unseen: &UnseenAuthoredEnvelope<'_>,
    ) -> Result<
        (
            HandlerResult<
                WorthQueryCompletedMutationCandidate<S, B>,
                <B as ApplicationMutationBinding<S>>::Denial,
            >,
            WorthQueryMutationHandlerWork,
        ),
        WorthQueryOutputDemandDenial,
    >
    where
        B::Input: Clone + Send + Sync,
        B::Result: Send,
        B::Denial: Send,
    {
        self.0
            .prepare_prefix_candidate::<B>(access, phase, application, unseen)
    }

    pub fn with_declared_output_dependencies<B: ApplicationMutationBinding<S>>(
        self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        self.0
            .with_declared_output_dependencies::<B>(application)
            .map(Self)
    }

    pub fn with_required_output_dependencies<B, D, E>(
        self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        demand: D,
        position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        order: worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    ) -> Result<Self, WorthQueryOutputDemandDenial>
    where
        B: ApplicationMutationBinding<S>,
        D: WorthQueryApplicationOutputDemand<S> + Clone + Send + Sync,
        E: super::producer::synchronous_outputs::SynchronousOutputEdges<S, D> + 'static,
        super::producer::synchronous_outputs::source::SourceInput<S, D>:
            worth_query_declaration::facade::application_query::ApplicationQueryIntent<
                S,
                Binding = super::producer::synchronous_outputs::source::Source<S, D>,
            >,
        <super::producer::synchronous_outputs::source::Source<S, D> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<S>>::ScopeBinding:
            worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution<
                S,
                <super::producer::synchronous_outputs::source::Source<S, D> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
{
        self.0
            .with_required_output_dependencies::<B, D, E>(application, demand, position, order)
            .map(Self)
    }
}

impl<S> CapturedAuthoredEditStage<S> {
    pub fn with_correction_intent(
        self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        intent: WorthQueryCorrectionIntent,
    ) -> Self {
        Self(self.0.with_correction_intent(access, intent))
    }

    pub fn same_admitted_basis(&self, other: &Self) -> bool {
        self.0.same_admitted_basis(&other.0)
    }

    pub fn subject(&self) -> DeclaredEditStage {
        self.0.subject()
    }
}

pub fn with_resolved_edit_envelope<S: ApplicationSchema, R>(
    access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    stages: &[CapturedAuthoredEditStage<S>],
    intent: ApplicationEditIntentIdentity,
    purpose: AuthoredEnvelopeResolutionPurpose,
    proceed: impl FnOnce(AuthoredEnvelopeResolution<'_>) -> R,
) -> Result<R, WorthQueryApplicationIdempotencyResolutionDenial> {
    let inner_stages: Vec<_> = stages.iter().map(|stage| stage.0.clone()).collect();
    inner::with_resolved_edit_envelope(
        access,
        phase,
        application,
        &inner_stages,
        intent,
        purpose,
        proceed,
    )
}

pub fn order_authored_stages<S: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stages: &mut [CapturedAuthoredEditStage<S>],
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    unseen: &UnseenAuthoredEnvelope<'_>,
) -> Result<Vec<usize>, DeclaredEditOrderRefusal> {
    let mut inner_stages: Vec<_> = stages.iter().map(|stage| stage.0.clone()).collect();
    let result =
        inner::order_authored_stages(execution, &mut inner_stages, phase, application, unseen);
    for (outer, updated) in stages.iter_mut().zip(inner_stages) {
        outer.0 = updated;
    }
    result
}
