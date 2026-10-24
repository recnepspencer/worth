//! Typed correction delivery reuses ordinary required-output custody.
use super::super::*;
use super::{binding::Inverse, candidate::NativePreparation, Prepare};
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission,
    performed::performed_outcome,
    performed_source::{require_program_output_root, PerformedSourceCommit},
};
use worth_query_declaration::facade::{
    application_aftermath::*, application_operation::*, application_program::*,
};
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::{
        WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    },
};
mod publication;
pub use publication::WorthQueryPreparedRequiredCorrection;
type Connection<S, R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
type Demand<S, R> = <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
type Outcome<S, B, P, R> =
    crate::application_entry::WorthQueryApplicationPerformedMutationOutcome<S, B, P, R>;

/// A required-output correction keeps the original delivery owner distinct from retry status.
/// An equivalent settled retry does not replace an output handle already held by its caller.
pub enum WorthQueryRequiredCorrectionPreparation<'a, S, B, P, R>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>
        + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    Prepared(WorthQueryPreparedRequiredCorrection<'a, S, B, P, R>),
    Settled(Outcome<S, B, P, R>),
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}
impl<'a, S: ApplicationSchema> WorthQueryApplicationCorrectionRequest<'a, '_, '_, '_, S> {
    /// Prepares the installed inverse with the program's typed required root.
    /// Publication returns the ordinary move-only performed/output recovery owners.
    pub fn prepare_performed<C, P, R>(
        self,
        application: &'a WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQueryRequiredCorrectionPreparation<'a, S, Inverse<S, C>, P, R>,
        WorthQueryApplicationCorrectionDenial,
    >
    where
        C: ApplicationCorrectionBinding<S>,
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        Inverse<S, C>: ApplicationMutationAdmissionBinding<S>
            + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
        <Inverse<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission:
            WorthQueryBindingAdmission<S, Inverse<S, C>>,
        <Inverse<S, C> as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, Inverse<S, C>>,
        <Inverse<S, C> as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <Inverse<S, C> as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        use WorthQueryApplicationCorrectionDenial as Denial;
        require_program_output_root::<S, P, R>(self.correction.request.application, application)
            .map_err(Denial::RequiredOutputs)?;
        let principal = self.correction.request.principal;
        let scope = self.correction.request.scope;
        let source = self.correction.source;
        let limits = self.limits;
        let runtime = self.correction.request.application;
        runtime.with_application_advancement(scope, |phase| {
            let mut demand = None;
            let prepared = Prepare { request: self, phase: &phase }
                .candidate::<C, <Inverse<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission>(|target, access, key| {
                    let (input, source) = target.derive_required_output_correction::<C, R>(
                        access, &phase, principal, scope, source, key,
                        limits.maximum_ancestors, limits.maximum_disclosure_results,
                    ).map_err(Denial::Derivation)?;
                    demand = Some(<Inverse<S, C> as WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>>::demand_from_source(input.input())
                        .map_err(|cause| Denial::RequiredOutputs(crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Connection(cause)))?);
                    Ok((input, source))
                })?;
            Ok(retain_required_output(prepared, application, demand))
        }).map_err(Denial::ExecutionRequest)?
    }
}

pub(super) fn retain_required_output<'a, S, B, P, R>(
    prepared: NativePreparation<'a, S, B>,
    application: &'a WorthQueryProgramApplicationRuntime<S, P>,
    demand: Option<Demand<S, R>>,
) -> WorthQueryRequiredCorrectionPreparation<'a, S, B, P, R>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>
        + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    match prepared {
        NativePreparation::Prepared(action) => WorthQueryRequiredCorrectionPreparation::Prepared(
            WorthQueryPreparedRequiredCorrection { action, application, demand: demand.expect("first-use derivation supplies its actual input demand") }),
        NativePreparation::IntentDrift => WorthQueryRequiredCorrectionPreparation::Settled(Outcome::NotPerformed(crate::application_entry::WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)),
        NativePreparation::Replay { outcome, .. } => WorthQueryRequiredCorrectionPreparation::Settled(Outcome::NotPerformed(outcome)),
        NativePreparation::PublicationPending => WorthQueryRequiredCorrectionPreparation::PublicationPending,
        NativePreparation::ProductUnpublished(handle) => WorthQueryRequiredCorrectionPreparation::ProductUnpublished(handle),
    }
}
