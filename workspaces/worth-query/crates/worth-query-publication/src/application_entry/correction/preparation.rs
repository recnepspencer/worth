//! Correction request progression keeps the native declaration's admission dispatch.
use super::*;
use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationCorrectionBinding, ApplicationCorrectionVisitor, ApplicationInverseBinding,
    },
    application_capability::{ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest},
    application_operation::*,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
mod binding;
mod operation;
mod required_output;
pub use operation::{
    WorthQueryOperationCorrectionPreparation, WorthQueryPreparedOperationCorrection,
};
pub use required_output::{
    WorthQueryPreparedRequiredCorrection, WorthQueryRequiredCorrectionPreparation,
};

/// Preparation either retains an unpublished inverse candidate or returns existing
/// command evidence and custody. A settled retry does not execute the inverse again.
pub enum WorthQueryApplicationCorrectionPreparation<'application> {
    Prepared(super::publication::WorthQueryPreparedApplicationCorrection<'application>),
    Settled(super::outcome::WorthQueryApplicationCorrectionOutcome),
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}
/// A recorded reversal bound to its client command key, awaiting explicit history limits.
pub struct WorthQueryKeyedApplicationCorrection<'a, 'p, 's, 'h, Schema> {
    correction: WorthQueryApplicationCorrection<'a, 'p, 's, 'h, Schema>,
    key: String,
}
/// A keyed reversal with declared lookup work bounds, ready for fresh inverse admission.
/// Preparation retains its captured input and exact target until publication or drop.
pub struct WorthQueryApplicationCorrectionRequest<'a, 'p, 's, 'h, Schema> {
    correction: WorthQueryApplicationCorrection<'a, 'p, 's, 'h, Schema>,
    key: String,
    limits: WorthQueryCorrectionHistoryLimits,
}
impl<'a, 'p, 's, 'h, Schema> WorthQueryApplicationCorrection<'a, 'p, 's, 'h, Schema> {
    pub fn idempotency(
        self,
        key: &str,
    ) -> WorthQueryKeyedApplicationCorrection<'a, 'p, 's, 'h, Schema> {
        WorthQueryKeyedApplicationCorrection {
            correction: self,
            key: key.to_owned(),
        }
    }
}
impl<'a, 'p, 's, 'h, Schema> WorthQueryKeyedApplicationCorrection<'a, 'p, 's, 'h, Schema> {
    pub fn limits(
        self,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> WorthQueryApplicationCorrectionRequest<'a, 'p, 's, 'h, Schema> {
        WorthQueryApplicationCorrectionRequest {
            correction: self.correction,
            key: self.key,
            limits,
        }
    }
}
impl<'a, Schema: ApplicationSchema> WorthQueryApplicationCorrectionRequest<'a, '_, '_, '_, Schema> {
    pub fn prepare(
        self,
    ) -> Result<WorthQueryApplicationCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    {
        let request = &self.correction.request;
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                Schema::visit_correction(
                    self.correction.source.source_binding_type(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                    ),
                    Prepare {
                        request: self,
                        phase: &phase,
                    },
                )
            })
            .map_err(WorthQueryApplicationCorrectionDenial::ExecutionRequest)?
    }
}
struct Prepare<'a, 'p, 's, 'h, 'phase, 'borrow, Schema> {
    request: WorthQueryApplicationCorrectionRequest<'a, 'p, 's, 'h, Schema>,
    phase: &'borrow WorthQueryAdvancementPhase<'phase>,
}
impl<'a, Schema: ApplicationSchema> ApplicationCorrectionVisitor<Schema>
    for Prepare<'a, '_, '_, '_, '_, '_, Schema>
{
    type Output = Result<
        WorthQueryApplicationCorrectionPreparation<'a>,
        WorthQueryApplicationCorrectionDenial,
    >;
    fn unavailable(self) -> Self::Output {
        Err(WorthQueryApplicationCorrectionDenial::SourceBindingUnavailable)
    }
    fn conventional<C: ApplicationCorrectionBinding<Schema>>(self) -> Self::Output
    where
        ApplicationInverseBinding<Schema, C>: ApplicationMutationAdmissionBinding<Schema, Admission = ApplicationConventionalAdmission>,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, C>>,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<Schema, <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    {
        self.prepare::<C, ApplicationConventionalAdmission>()
    }
    fn capability<C, Capability>(self) -> Self::Output
    where
        C: ApplicationCorrectionBinding<Schema>,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        ApplicationInverseBinding<Schema, C>: ApplicationCapabilityMutationBinding<Schema, Capability = Capability>,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, C>> + ApplicationCapabilityRequest<Schema, Capability, Scope = <<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope>,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<Schema, <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    {
        self.prepare::<C, ApplicationCapabilityAdmission<Capability>>()
    }
}
