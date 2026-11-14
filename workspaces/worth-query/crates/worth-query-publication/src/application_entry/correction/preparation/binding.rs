//! One ordinary inverse admission and handler spine, shared by declared admission modes.
use super::candidate::NativePreparation;
use super::*;
use crate::application_entry::mutation::authorization::WorthQueryBindingAdmission;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;
use worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionActionInputSelection;

pub(super) type Inverse<S, C> = ApplicationInverseBinding<S, C>;

impl<'a, Schema: ApplicationSchema> Prepare<'a, '_, '_, '_, '_, '_, Schema> {
    pub(super) fn candidate<C, Admission>(
        self,
        derive: impl FnOnce(
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'_, Schema>,
            &worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess,
            &worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionCommandKey,
        ) -> Result<(
            ApplicationEncodedInput<<Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>,
            std::sync::Arc<worth_query_execution::publication_boundary::recorded_correction::WorthQueryPreparedCorrectionSource>,
        ), WorthQueryApplicationCorrectionDenial>,
    ) -> Result<
        NativePreparation<'a, Schema, Inverse<Schema, C>>,
        WorthQueryApplicationCorrectionDenial,
    >
    where
        C: ApplicationCorrectionBinding<Schema>,
        Admission: WorthQueryBindingAdmission<Schema, Inverse<Schema, C>>,
        <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<Schema, Inverse<Schema, C>>,
        <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        self.with_input::<C, _>(
            false,
            derive,
            |request, target, key, command_key, selected, phase| {
                super::candidate::prepare_action_candidate::<Schema, Inverse<Schema, C>, Admission>(
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
            super::candidate::CorrectionFront::Selected(prepared) => prepared,
            super::candidate::CorrectionFront::IntentDrift => NativePreparation::IntentDrift,
            super::candidate::CorrectionFront::PublicationPending => {
                NativePreparation::PublicationPending
            }
            super::candidate::CorrectionFront::ProductUnpublished(handle) => {
                NativePreparation::ProductUnpublished(handle)
            }
        })
    }

    pub(super) fn with_input<C, T>(
        self,
        envelope_identity: bool,
        derive: impl FnOnce(
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, Schema>,
            &worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess,
            &worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionCommandKey,
        ) -> Result<(
            ApplicationEncodedInput<<Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>,
            std::sync::Arc<worth_query_execution::publication_boundary::recorded_correction::WorthQueryPreparedCorrectionSource>,
        ), WorthQueryApplicationCorrectionDenial>,
        next: impl FnOnce(
            WorthQueryApplicationCorrections<'a, '_, '_, Schema>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, Schema>,
            <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::IdempotencyKey,
            worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionCommandKey,
            super::candidate::CorrectionActionInput<Schema, Inverse<Schema, C>>,
            &WorthQueryAdvancementPhase<'_>,
        ) -> Result<T, WorthQueryApplicationCorrectionDenial>,
    ) -> Result<super::candidate::CorrectionFront<T>, WorthQueryApplicationCorrectionDenial>
    where
        C: ApplicationCorrectionBinding<Schema>,
        <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<Schema, Inverse<Schema, C>>,
        <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        use super::candidate::CorrectionFront as Preparation;
        use WorthQueryApplicationCorrectionDenial as Denial;
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
            .correction_command_key::<C>(
                &access,
                phase,
                request.principal,
                request.scope,
                &self.request.key,
            )
            .map_err(Denial::Derivation)?;
        let inverse_key = C::inverse_key(&self.request.key).map_err(|cause| {
            Denial::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(
                ApplicationMutationIdentityDenial::Key(cause),
            ))
        })?;
        let selected = target
            .select_correction_input::<C>(
                &access,
                phase,
                request.principal,
                request.scope,
                correction.source,
                &command_key,
                self.request.limits.maximum_ancestors,
                self.request.limits.maximum_disclosure_results,
            )
            .map_err(Denial::Derivation)?;
        let (input, source, intent) = match selected {
            WorthQueryCorrectionActionInputSelection::FirstUse(_) => {
                let (input, source) = derive(&target, &access, &command_key)?;
                let intent = source.intent();
                (input, Some(source), intent)
            }
            WorthQueryCorrectionActionInputSelection::Replay { intent, input } => {
                (input, None, intent)
            }
            WorthQueryCorrectionActionInputSelection::IntentDrift => {
                return Ok(Preparation::IntentDrift)
            }
            WorthQueryCorrectionActionInputSelection::PublicationPending if !envelope_identity => {
                return Ok(Preparation::PublicationPending)
            }
            WorthQueryCorrectionActionInputSelection::ProductUnpublished(handle)
                if !envelope_identity =>
            {
                return Ok(Preparation::ProductUnpublished(handle))
            }
            WorthQueryCorrectionActionInputSelection::PublicationPending
            | WorthQueryCorrectionActionInputSelection::ProductUnpublished(_) => {
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
            inverse_key,
            command_key,
            super::candidate::CorrectionActionInput {
                input,
                source,
                intent,
            },
            phase,
        )
        .map(Preparation::Selected)
    }
}

impl<'a, Schema: ApplicationSchema> Prepare<'a, '_, '_, '_, '_, '_, Schema> {
    pub(super) fn prepare<C, Admission>(
        self,
    ) -> Result<WorthQueryApplicationCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    where
        C: ApplicationCorrectionBinding<Schema>,
        Admission: WorthQueryBindingAdmission<Schema, Inverse<Schema, C>>,
        <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<Schema, Inverse<Schema, C>>,
        <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Inverse<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let principal = self.request.correction.request.principal;
        let scope = self.request.correction.request.scope;
        let source = self.request.correction.source;
        let limits = self.request.limits;
        let phase = self.phase;
        let prepared = self.candidate::<C, Admission>(|target, access, key| {
            target
                .derive_correction_inverse::<C>(
                    access,
                    phase,
                    principal,
                    scope,
                    source,
                    key,
                    limits.maximum_ancestors,
                    limits.maximum_disclosure_results,
                )
                .map_err(WorthQueryApplicationCorrectionDenial::Derivation)
        })?;
        Ok(prepared.into_public())
    }
}
