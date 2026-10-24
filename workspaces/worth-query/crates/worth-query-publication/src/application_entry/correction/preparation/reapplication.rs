//! Reapply retained source meaning through fresh ordinary action admission.
use super::*;
use worth_query_declaration::facade::application_aftermath::{
    dispatch_application_reapplication, ApplicationReapplicationVisitor,
};
use worth_query_execution::facade::primary_graph::WorthQuerySelectedCorrectionAlternative;
mod binding;
mod operation;
mod required_output;

/// A selected historical alternative bound to this request's principal and target.
pub struct WorthQueryApplicationReapplication<'a, 'p, 's, 'h, S> {
    pub(super) request: WorthQueryApplicationCorrections<'a, 'p, 's, S>,
    pub(super) alternative: &'h WorthQuerySelectedCorrectionAlternative<'a, S>,
}

/// A reapplication command awaiting explicit history lookup bounds.
pub struct WorthQueryKeyedApplicationReapplication<'a, 'p, 's, 'h, S> {
    correction: WorthQueryApplicationReapplication<'a, 'p, 's, 'h, S>,
    key: String,
}

/// A retained-meaning command ready for fresh source admission and preparation.
pub struct WorthQueryApplicationReapplicationRequest<'a, 'p, 's, 'h, S> {
    pub(super) correction: WorthQueryApplicationReapplication<'a, 'p, 's, 'h, S>,
    pub(super) key: String,
    pub(super) limits: WorthQueryCorrectionHistoryLimits,
}

impl<'a, 'p, 's, S: ApplicationSchema> WorthQueryApplicationCorrections<'a, 'p, 's, S> {
    /// Uses the alternative's retained original input under this fresh request.
    pub fn reapply<'h>(
        self,
        alternative: &'h WorthQuerySelectedCorrectionAlternative<'a, S>,
    ) -> WorthQueryApplicationReapplication<'a, 'p, 's, 'h, S> {
        WorthQueryApplicationReapplication {
            request: self,
            alternative,
        }
    }
}

impl<'a, 'p, 's, 'h, S> WorthQueryApplicationReapplication<'a, 'p, 's, 'h, S> {
    pub fn idempotency(
        self,
        key: &str,
    ) -> WorthQueryKeyedApplicationReapplication<'a, 'p, 's, 'h, S> {
        WorthQueryKeyedApplicationReapplication {
            correction: self,
            key: key.to_owned(),
        }
    }
}

impl<'a, 'p, 's, 'h, S> WorthQueryKeyedApplicationReapplication<'a, 'p, 's, 'h, S> {
    pub fn limits(
        self,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> WorthQueryApplicationReapplicationRequest<'a, 'p, 's, 'h, S> {
        WorthQueryApplicationReapplicationRequest {
            correction: self.correction,
            key: self.key,
            limits,
        }
    }
}

impl<'a, S: ApplicationSchema> WorthQueryApplicationReapplicationRequest<'a, '_, '_, '_, S> {
    /// Admits the installed source binding using its retained original input.
    pub fn prepare(
        self,
    ) -> Result<WorthQueryApplicationCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    {
        let request = &self.correction.request;
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                S::visit_correction(
                    self.correction.alternative.source_binding_type(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                    ),
                    PrepareReapplication {
                        request: self,
                        phase: &phase,
                    },
                )
            })
            .map_err(WorthQueryApplicationCorrectionDenial::ExecutionRequest)?
    }
}

pub(super) struct PrepareReapplication<'a, 'p, 's, 'h, 'phase, 'borrow, S> {
    pub(super) request: WorthQueryApplicationReapplicationRequest<'a, 'p, 's, 'h, S>,
    pub(super) phase: &'borrow WorthQueryAdvancementPhase<'phase>,
}

impl<'a, S: ApplicationSchema> ApplicationCorrectionVisitor<S>
    for PrepareReapplication<'a, '_, '_, '_, '_, '_, S>
{
    type Output = Result<
        WorthQueryApplicationCorrectionPreparation<'a>,
        WorthQueryApplicationCorrectionDenial,
    >;
    fn unavailable(self) -> Self::Output {
        Err(WorthQueryApplicationCorrectionDenial::SourceBindingUnavailable)
    }
    fn declared<C: ApplicationCorrectionBinding<S>>(self) -> Self::Output {
        dispatch_application_reapplication::<S, C, _>(self)
    }
}

impl<'a, S: ApplicationSchema> ApplicationReapplicationVisitor<S>
    for PrepareReapplication<'a, '_, '_, '_, '_, '_, S>
{
    type Output = Result<
        WorthQueryApplicationCorrectionPreparation<'a>,
        WorthQueryApplicationCorrectionDenial,
    >;
    fn unavailable(self) -> Self::Output {
        Err(WorthQueryApplicationCorrectionDenial::SourceBindingUnavailable)
    }
    fn conventional<C: ApplicationCorrectionBinding<S>>(self) -> Self::Output
    where
        C::Source:
            ApplicationMutationAdmissionBinding<S, Admission = ApplicationConventionalAdmission>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        self.prepare::<C, ApplicationConventionalAdmission>()
    }

    fn capability<C, Capability>(self) -> Self::Output
    where
        C: ApplicationCorrectionBinding<S>,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = S> + 'static,
        C::Source: ApplicationCapabilityMutationBinding<S, Capability = Capability>,
        <C::Source as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source> + ApplicationCapabilityRequest<S, Capability, Scope = <<C::Source as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    {
        self.prepare::<C, ApplicationCapabilityAdmission<Capability>>()
    }
}
