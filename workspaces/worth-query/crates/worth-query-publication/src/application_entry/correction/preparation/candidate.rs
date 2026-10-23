//! Ordinary admission and candidate preparation for the selected correction action.
use super::*;
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission,
    captured_input::CapturedMutationInput,
    preparation::{CandidatePreparation, PreparedCandidate},
    request::WorthQueryApplicationMutationRequest,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;
use std::sync::Arc;
use worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_execution::publication_boundary::recorded_correction::{
    WorthQueryCorrectionCommandKey, WorthQueryCorrectionIntent, WorthQueryPreparedCorrectionSource,
};

pub(super) enum NativePreparation<'a, S: ApplicationSchema, B: ApplicationMutationBinding<S>> {
    Prepared(super::super::publication::PreparedCorrectionAction<'a, S, B>),
    IntentDrift,
    Replay {
        outcome:
            crate::application_entry::WorthQueryApplicationMutationOutcome<B::Denial, B::Result>,
        identities: ApplicationOwnedMutationIdentities<S, B>,
        intent: WorthQueryCorrectionIntent,
    },
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}

impl<'a, S: ApplicationSchema, B: ApplicationMutationBinding<S>> NativePreparation<'a, S, B>
where
    B::Input: Clone + Send + Sync,
{
    pub(super) fn into_public(self) -> WorthQueryApplicationCorrectionPreparation<'a> {
        use WorthQueryApplicationCorrectionPreparation as Public;
        match self {
            Self::Prepared(prepared) => Public::Prepared(
                super::super::publication::WorthQueryPreparedApplicationCorrection {
                    publication: Box::new(prepared),
                },
            ),
            Self::IntentDrift => Public::Settled(crate::application_entry::WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift),
            Self::Replay { outcome, .. } => Public::Settled(super::super::outcome::erase::<S, B>(outcome)),
            Self::PublicationPending => Public::PublicationPending,
            Self::ProductUnpublished(handle) => Public::ProductUnpublished(handle),
        }
    }
}

/// Original encoded action input and native first-use or retry evidence.
pub(super) struct CorrectionActionInput<
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    pub(super) input: ApplicationEncodedInput<Binding::InputBinding>,
    pub(super) source: Option<Arc<WorthQueryPreparedCorrectionSource>>,
    pub(super) intent: WorthQueryCorrectionIntent,
}

pub(super) fn prepare_action_candidate<'a, Schema, Binding, Admission>(
    request: WorthQueryApplicationCorrections<'a, '_, '_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    target: &WorthQuerySelectedProductOperation<'a, Schema>,
    key: Binding::IdempotencyKey,
    command_key: WorthQueryCorrectionCommandKey,
    selected: CorrectionActionInput<Schema, Binding>,
) -> Result<NativePreparation<'a, Schema, Binding>, WorthQueryApplicationCorrectionDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Admission: WorthQueryBindingAdmission<Schema, Binding>,
    Binding::Input: Clone + Send + Sync + ApplicationMutationInputScope<Schema, Binding>,
    Binding::ScopeBinding: ApplicationMutationScopeResolution<Schema, Binding::PrincipalIdentity>,
{
    use NativePreparation as Preparation;
    use WorthQueryApplicationCorrectionDenial as Denial;
    let CorrectionActionInput {
        input,
        source,
        intent,
    } = selected;
    let identities =
        ApplicationOwnedMutationIdentities::<Schema, Binding>::with_encoded_input(key, &input)
            .map_err(|cause| {
                Denial::Mutation(WorthQueryApplicationRequestMutationDenial::Identity(cause))
            })?
            .with_additional_canonical_work(command_key.canonical().work());
    let mutation = WorthQueryApplicationMutationRequest::new(
        request.application,
        request.principal,
        request.scope,
        request.branch,
        CapturedMutationInput::<Schema, Binding>::new(input),
    );
    let mut mutation = mutation.idempotency(identities.key());
    let Some(source) = source else {
        let staged = mutation.stage().map_err(Denial::Mutation)?;
        let prepared =
            Admission::prepare_selected(phase, &mutation, &identities.borrow(), staged, target)
                .map_err(Denial::Mutation)?;
        let prepared = bind_correction(prepared, intent);
        let outcome = mutation
            .resolve_idempotency(phase, &prepared.admission, prepared.idempotency)
            .map_err(Denial::Mutation)?
            .ok_or(Denial::RecordedOutcomeUnavailable)?;
        return Ok(Preparation::Replay {
            outcome,
            identities,
            intent,
        });
    };
    let owner = request
        .application
        .selected_program_owner(target)
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
                Admission::prepare_selected(phase, mutation, identities, staged, target)
                    .map(|prepared| bind_correction(prepared, intent))
            },
            &mut work,
            execution.allocation_policy(),
        )
        .map_err(Denial::Mutation)?;
    match candidate {
        CandidatePreparation::Settled(outcome) => Ok(Preparation::Replay {
            outcome,
            identities,
            intent,
        }),
        CandidatePreparation::Prepared(PreparedCandidate {
            program, result, ..
        }) => Ok(Preparation::Prepared(
            super::super::publication::PreparedCorrectionAction {
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

fn bind_correction<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>(
    mut prepared: crate::application_entry::mutation::authorization::PreparedMutation<
        Schema,
        Binding,
    >,
    intent: WorthQueryCorrectionIntent,
) -> crate::application_entry::mutation::authorization::PreparedMutation<Schema, Binding> {
    prepared.extension.correction = Some(intent);
    prepared.idempotency = intent.bind_idempotency(prepared.idempotency);
    prepared
}
