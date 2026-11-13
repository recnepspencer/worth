//! Whole-envelope inspection stays governed by every real nominal admission.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::idempotency::envelope_candidates::EnvelopeCandidates;
use crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination;
use worth_query_declaration::facade::application_operation::ApplicationEditIntentIdentity;
pub(super) type Denial =
    crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial;
pub(super) type Resolution =
    crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution;

/// Request-local Unseen custody. Construction and its serialization stay at Query.
pub struct UnseenAuthoredEnvelope<'a> {
    pub(super) intent: ApplicationEditIntentIdentity,
    pub(in crate::domain_computation::primary_graph) coordination:
        &'a WorthQueryApplicationBranchCommitCoordination<'a>,
    bindings: &'a [WorthQueryApplicationIdempotencyBinding],
}
impl UnseenAuthoredEnvelope<'_> {
    pub(in crate::domain_computation::primary_graph) fn admits(
        &self,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> bool {
        self.bindings
            .iter()
            .any(|candidate| candidate.has_same_intent(binding))
    }
}
pub enum AuthoredEnvelopeResolution<'a> {
    Unseen(UnseenAuthoredEnvelope<'a>),
    Recorded(crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolution),
}

/// The callback cannot retain a coordination grant after the owning lane closes.
pub fn with_resolved_edit_envelope<S: ApplicationSchema, R>(
    _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    phase: &WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    stages: &[CapturedAuthoredEditStage<S>],
    intent: ApplicationEditIntentIdentity,
    proceed: impl FnOnce(AuthoredEnvelopeResolution<'_>) -> R,
) -> Result<R, Denial> {
    let first = stages.first().ok_or_else(Denial::foreign_admission)?;
    if stages.iter().enumerate().any(|(index, stage)| {
        !stage.same_admitted_basis(first)
            || stages[..index]
                .iter()
                .any(|earlier| earlier.identity == stage.identity)
    }) {
        return Err(Denial::foreign_admission());
    }
    let context = AuthoredEditStageContext { phase, application };
    let product = first.executor.product(&context)?;
    if product.observation() != &first.before {
        return Err(Denial::foreign_admission());
    }
    let execution = phase
        .checked_execution_request_for(application.product_runtime())
        .map_err(|_| Denial::foreign_admission())?;
    let lane = application
        .runtime
        .primary_provider
        .application_branch_commit_lane(product.observation());
    let coordination = lane.enter();
    let capacity = || {
        Denial::from_provider(crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted)
    };
    let bytes = stages
        .len()
        .checked_mul(size_of::<WorthQueryApplicationIdempotencyBinding>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(capacity)?;
    let _memory = execution
        .allocation_policy()
        .reserve_memory(bytes)
        .map_err(|_| capacity())?;
    let mut bindings = Vec::new();
    bindings
        .try_reserve_exact(stages.len())
        .map_err(|_| capacity())?;
    if bindings.capacity() != stages.len() {
        return Err(capacity());
    }
    let resolution = govern(
        stages,
        &context,
        &coordination,
        intent,
        &mut bindings,
        &mut |bindings| {
            let candidates = EnvelopeCandidates::new(bindings).map_err(Denial::from_provider)?;
            application
                .runtime
                .primary_provider
                .resolve_envelope_idempotency_at_product(
                    &execution,
                    &candidates,
                    &product.publication_binding(),
                )
                .map_err(Denial::from_provider)
        },
    )?;
    if matches!(resolution, Resolution::Absent) {
        return Ok(proceed(AuthoredEnvelopeResolution::Unseen(
            UnseenAuthoredEnvelope {
                intent,
                coordination: &coordination,
                bindings: &bindings,
            },
        )));
    }
    let resolution = application.envelope_resolution(resolution, &product.publication_binding())?;
    Ok(proceed(AuthoredEnvelopeResolution::Recorded(resolution)))
}

fn govern<S: ApplicationSchema>(
    stages: &[CapturedAuthoredEditStage<S>],
    context: &AuthoredEditStageContext<'_, '_, '_, S>,
    coordination: &WorthQueryApplicationBranchCommitCoordination<'_>,
    intent: ApplicationEditIntentIdentity,
    bindings: &mut Vec<WorthQueryApplicationIdempotencyBinding>,
    resolve: &mut dyn FnMut(
        &[WorthQueryApplicationIdempotencyBinding],
    ) -> Result<Resolution, Denial>,
) -> Result<Resolution, Denial> {
    let Some((first, rest)) = stages.split_first() else {
        return resolve(bindings);
    };
    first
        .executor
        .govern_envelope(context, coordination, intent, &mut |binding| {
            bindings.push(binding);
            // Each nominal inspection proof remains live around the shared read.
            govern(rest, context, coordination, intent, bindings, resolve)
        })
}
