//! One ordinary inverse admission and handler spine, shared by declared admission modes.
use super::*;
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission,
    captured_input::CapturedMutationInput,
    preparation::{CandidatePreparation, PreparedCandidate},
    request::WorthQueryApplicationMutationRequest,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;
use worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionInputSelection;

pub(super) type Inverse<S, C> = ApplicationInverseBinding<S, C>;

pub(super) enum NativePreparation<'a, S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    Prepared(super::super::publication::PreparedInverse<'a, S, B>),
    Settled(crate::application_entry::WorthQueryApplicationMutationOutcome<B::Denial, B::Result>),
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}

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
        use NativePreparation as Preparation;
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
            WorthQueryCorrectionInputSelection::FirstUse(_) => {
                let (input, source) = derive(&target, &access, &command_key)?;
                let intent = source.intent();
                (input, Some(source), intent)
            }
            WorthQueryCorrectionInputSelection::Replay { intent, input } => (input, None, intent),
            WorthQueryCorrectionInputSelection::IntentDrift => return Ok(Preparation::Settled(
                crate::application_entry::WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)),
            WorthQueryCorrectionInputSelection::PublicationPending => return Ok(Preparation::PublicationPending),
            WorthQueryCorrectionInputSelection::ProductUnpublished(handle) => return Ok(Preparation::ProductUnpublished(handle)),
        };
        let identities =
            ApplicationOwnedMutationIdentities::<Schema, Inverse<Schema, C>>::with_encoded_input(
                C::inverse_key(&self.request.key),
                &input,
            )
            .map_err(|cause| {
                Denial::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(cause))
            })?
            .with_additional_canonical_work(command_key.canonical().work());
        let mutation = WorthQueryApplicationMutationRequest::new(
            request.application,
            request.principal,
            request.scope,
            request.branch,
            CapturedMutationInput::<Schema, Inverse<Schema, C>>::new(input),
        );
        let mut mutation = mutation.idempotency(identities.key());
        let Some(source) = source else {
            let staged = mutation.stage().map_err(Denial::Mutation)?;
            let prepared = Admission::prepare_selected(
                phase,
                &mutation,
                &identities.borrow(),
                staged,
                &target,
            )
            .map_err(Denial::Mutation)?;
            let prepared = bind_correction(prepared, intent);
            let outcome = mutation
                .resolve_idempotency(phase, &prepared.admission, prepared.idempotency)
                .map_err(Denial::Mutation)?
                .ok_or(Denial::RecordedOutcomeUnavailable)?;
            return Ok(Preparation::Settled(outcome));
        };
        let owner = request
            .application
            .selected_program_owner(&target)
            .map_err(Denial::ProgramOwner)?;
        let execution = phase
            .checked_execution_request_for(request.application.product_runtime())
            .map_err(Denial::ExecutionRequest)?;
        let mut work =
            worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted;
        let candidate = mutation
            .prepare_owned_candidate(
                phase,
                identities.borrow(),
                |mutation, identities, staged| {
                    Admission::prepare_selected(phase, mutation, identities, staged, &target)
                        .map(|prepared| bind_correction(prepared, intent))
                },
                &mut work,
                execution.allocation_policy(),
            )
            .map_err(Denial::Mutation)?;
        match candidate {
            CandidatePreparation::Settled(outcome) => Ok(Preparation::Settled(outcome)),
            CandidatePreparation::Prepared(PreparedCandidate {
                program, result, ..
            }) => Ok(Preparation::Prepared(
                super::super::publication::PreparedInverse::<Schema, Inverse<Schema, C>> {
                    application: request.application,
                    owner,
                    program,
                    identities,
                    result,
                    source,
                },
            )),
        }
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
        use WorthQueryApplicationCorrectionPreparation as Public;
        Ok(match prepared {
            NativePreparation::Prepared(prepared) => Public::Prepared(
                super::super::publication::WorthQueryPreparedApplicationCorrection {
                    publication: Box::new(prepared),
                },
            ),
            NativePreparation::Settled(outcome) => Public::Settled(super::super::outcome::erase::<
                Schema,
                Inverse<Schema, C>,
            >(outcome)),
            NativePreparation::PublicationPending => Public::PublicationPending,
            NativePreparation::ProductUnpublished(handle) => Public::ProductUnpublished(handle),
        })
    }
}

fn bind_correction<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    mut prepared: crate::application_entry::mutation::authorization::PreparedMutation<
        Schema,
        Binding,
    >,
    intent: worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionIntent,
) -> crate::application_entry::mutation::authorization::PreparedMutation<Schema, Binding> {
    prepared.extension.correction = Some(intent);
    prepared.idempotency = intent.bind_idempotency(prepared.idempotency);
    prepared
}
