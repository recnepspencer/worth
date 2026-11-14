//! Resolve original source input and use the shared ordinary correction candidate.
use super::super::candidate::{prepare_action_candidate, CorrectionActionInput, NativePreparation};
use super::*;
use crate::application_entry::mutation::authorization::WorthQueryBindingAdmission;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;
use worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionActionInputSelection;

impl<'a, S: ApplicationSchema> PrepareReapplication<'a, '_, '_, '_, '_, '_, S> {
    pub(in super::super) fn candidate<C, Admission>(
        self,
        derive: impl FnOnce(
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, S>,
            &worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess,
            &worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionCommandKey,
        ) -> Result<(
            ApplicationEncodedInput<<C::Source as ApplicationMutationBinding<S>>::InputBinding>,
            std::sync::Arc<worth_query_execution::publication_boundary::recorded_correction::WorthQueryPreparedCorrectionSource>,
        ), WorthQueryApplicationCorrectionDenial>,
    ) -> Result<NativePreparation<'a, S, C::Source>, WorthQueryApplicationCorrectionDenial>
    where
        C: ApplicationCorrectionBinding<S>,
        Admission: WorthQueryBindingAdmission<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        self.with_input::<C, _>(
            false,
            derive,
            |request, target, key, command_key, selected, phase| {
                prepare_action_candidate::<S, C::Source, Admission>(
                    request,
                    phase,
                    &target,
                    key,
                    command_key,
                    selected,
                )
            },
        )
        .map(|front| match front {
            super::super::candidate::CorrectionFront::Selected(prepared) => prepared,
            super::super::candidate::CorrectionFront::IntentDrift => NativePreparation::IntentDrift,
            super::super::candidate::CorrectionFront::PublicationPending => {
                NativePreparation::PublicationPending
            }
            super::super::candidate::CorrectionFront::ProductUnpublished(handle) => {
                NativePreparation::ProductUnpublished(handle)
            }
        })
    }

    pub(in super::super) fn with_input<C, T>(
        self,
        envelope_identity: bool,
        derive: impl FnOnce(
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, S>,
            &worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess,
            &worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionCommandKey,
        ) -> Result<(
            ApplicationEncodedInput<<C::Source as ApplicationMutationBinding<S>>::InputBinding>,
            std::sync::Arc<worth_query_execution::publication_boundary::recorded_correction::WorthQueryPreparedCorrectionSource>,
        ), WorthQueryApplicationCorrectionDenial>,
        next: impl FnOnce(
            WorthQueryApplicationCorrections<'a, '_, '_, S>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, S>,
            <C::Source as ApplicationMutationBinding<S>>::IdempotencyKey,
            worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionCommandKey,
            super::super::candidate::CorrectionActionInput<S, C::Source>,
            &WorthQueryAdvancementPhase<'_>,
        ) -> Result<T, WorthQueryApplicationCorrectionDenial>,
    ) -> Result<super::super::candidate::CorrectionFront<T>, WorthQueryApplicationCorrectionDenial>
    where
        C: ApplicationCorrectionBinding<S>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        use super::super::candidate::CorrectionFront as Front;
        use WorthQueryApplicationCorrectionDenial as Denial;
        use WorthQueryCorrectionActionInputSelection as Selected;
        let correction = self.request.correction;
        let request = correction.request;
        let phase = self.phase;
        let access = worth_query_execution::publication_boundary::program_publication_access();
        let target = request
            .application
            .on_branch(request.branch)
            .select(phase)
            .map_err(Denial::ProductSelection)?;
        let command_key = target
            .reapplication_command_key::<C>(
                &access,
                phase,
                request.principal,
                request.scope,
                &self.request.key,
            )
            .map_err(Denial::Derivation)?;
        let key = C::reapplication_key(&self.request.key).map_err(|cause| {
            Denial::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(
                ApplicationMutationIdentityDenial::Key(cause),
            ))
        })?;
        let selected = target
            .select_reapplication_input::<C>(
                &access,
                phase,
                request.principal,
                request.scope,
                correction.alternative,
                &command_key,
                self.request.limits.maximum_ancestors,
                self.request.limits.maximum_disclosure_results,
            )
            .map_err(Denial::Derivation)?;
        let (input, source, intent) = match selected {
            Selected::FirstUse(_) => {
                let (input, source) = derive(&target, &access, &command_key)?;
                let intent = source.intent();
                (input, Some(source), intent)
            }
            Selected::Replay { intent, input } => (input, None, intent),
            Selected::IntentDrift => return Ok(Front::IntentDrift),
            Selected::PublicationPending if !envelope_identity => {
                return Ok(Front::PublicationPending)
            }
            Selected::ProductUnpublished(handle) if !envelope_identity => {
                return Ok(Front::ProductUnpublished(handle))
            }
            Selected::PublicationPending | Selected::ProductUnpublished(_) => {
                // Identity derivation performs no handler work. Only the whole
                // authenticated envelope resolver may classify pending/recovery.
                let (input, source) = derive(&target, &access, &command_key)?;
                let intent = source.intent();
                (input, Some(source), intent)
            }
        };
        next(
            request,
            target,
            key,
            command_key,
            CorrectionActionInput {
                input,
                source,
                intent,
            },
            phase,
        )
        .map(Front::Selected)
    }

    pub(super) fn prepare<C, Admission>(
        self,
    ) -> Result<WorthQueryApplicationCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    where
        C: ApplicationCorrectionBinding<S>,
        Admission: WorthQueryBindingAdmission<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        let principal = self.request.correction.request.principal;
        let scope = self.request.correction.request.scope;
        let alternative = self.request.correction.alternative;
        let limits = self.request.limits;
        let phase = self.phase;
        let prepared = self.candidate::<C, Admission>(|target, access, key| {
            target
                .derive_correction_reapplication::<C>(
                    access,
                    phase,
                    principal,
                    scope,
                    alternative,
                    key,
                    limits.maximum_ancestors,
                    limits.maximum_disclosure_results,
                )
                .map_err(WorthQueryApplicationCorrectionDenial::Derivation)
        })?;
        Ok(prepared.into_public())
    }
}
