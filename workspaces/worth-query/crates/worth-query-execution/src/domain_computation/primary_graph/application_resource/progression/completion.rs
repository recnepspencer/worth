//! Authenticate and admit the typed scalar before genuine Native stage/commit.
use super::super::{
    occurrence::{ResourceCandidate, ResourceCandidateQueue},
    WorthQueryResourceFrameKind,
};
use super::*;

pub(super) enum Received {
    None,
    Committed,
    EffectsIndeterminate,
}

pub(super) fn receive<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &InstalledResourceBinding<Schema, Binding>,
    row: &mut ResourceRowLease,
    effect_authority: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecoveryInspectAuthority,
    >,
) -> Result<Received, ResourceCause>
where
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
{
    if row.occurrence().result.is_some() {
        return Ok(Received::Committed);
    }
    let key = row.key();
    let occurrence = row.occurrence_mut();
    let Some(index) = occurrence.native_slot else {
        return Ok(Received::None);
    };
    let Some(attempt) = occurrence.attempt.as_ref() else {
        return Ok(Received::None);
    };
    if occurrence.candidate.is_none() {
        let frame = installed.storage.nodes[index]
            .ingress
            .take_frame(attempt)
            .map_err(ResourceCause::Ingress)?;
        let Some(frame) = frame else {
            return Ok(Received::None);
        };
        occurrence.candidate = Some(ResourceCandidate {
            frame,
            verified: None,
            queue: ResourceCandidateQueue::Unadmitted,
            admission: None,
            validated: false,
            result_slot: None,
        });
    }
    let candidate = occurrence
        .candidate
        .as_mut()
        .expect("buffered candidate retained in actual row");
    let ResourceLifecycle::Live { owner, completion } = &mut occurrence.lifecycle else {
        return Err(ResourceCause::Unavailable);
    };
    if matches!(candidate.queue, ResourceCandidateQueue::Unadmitted) {
        let admitted = runtime
            .bridge
            .conditional()
            .enqueue_managed_owned_async_queue(owner, 1)
            .map_err(ResourceCause::Managed)?;
        candidate.queue = ResourceCandidateQueue::Admitted(admitted);
    }
    if candidate.verified.is_none() {
        let scratch = candidate
            .frame
            .copy(execution)
            .map_err(ResourceCause::Scratch)?;
        let verified = super::super::value_capture::verify_frame(
            execution,
            installed
                .installed
                .definition()
                .parts()
                .limits
                .maximum_frame_bytes,
            || installed.provider.verify(attempt, &scratch),
        )
        .map_err(ResourceCause::Value)?;
        candidate.verified = Some(verified);
    }
    let (kind, value) = candidate
        .verified
        .as_ref()
        .expect("verified projection retained once");
    if !candidate.validated {
        match kind {
            WorthQueryResourceFrameKind::Result => super::super::value_capture::validate_scalar::<
                Binding::ResultBinding,
            >(execution, &value.value),
            WorthQueryResourceFrameKind::DomainFailure
            | WorthQueryResourceFrameKind::EffectsIndeterminate => {
                super::super::value_capture::validate_scalar::<Binding::FailureBinding>(
                    execution,
                    &value.value,
                )
            }
        }
        .map_err(ResourceCause::Value)?;
        candidate.validated = true;
    }
    if *kind == WorthQueryResourceFrameKind::EffectsIndeterminate {
        let handle = occurrence
            .recovery
            .as_ref()
            .ok_or(ResourceCause::RecoveryHandoffRequired)?;
        let authority = effect_authority.ok_or(ResourceCause::RecoveryHandoffRequired)?;
        // The arbitrary verifier already ran. Re-observe the actual current
        // terminal owner now; original outbox provenance is a separate read.
        super::super::recovery::require_unresolved(
            runtime,
            execution,
            handle,
            authority,
            &occurrence.access,
            &occurrence.business_product,
        )?;
        if candidate.admission.is_none() {
            let witness = owner.request().ok_or(ResourceCause::Unavailable)?;
            let bytes =
                u64::try_from(candidate.frame.length()).map_err(|_| ResourceCause::Unavailable)?;
            let issued = witness.effects_indeterminate_issuer().certify(bytes);
            candidate.admission = Some(
                runtime
                    .bridge
                    .conditional()
                    .admit_owned_async_effects_indeterminate(issued)
                    .map_err(ResourceCause::Completion)?,
            );
        }
        let admission = candidate
            .admission
            .as_ref()
            .expect("actual indeterminate admission retained");
        if admission.report().denied_completion().is_some() {
            runtime
                .bridge
                .conditional()
                .release_process_local_completion_denial(owner, admission)
                .map_err(ResourceCause::Managed)?;
            return Err(ResourceCause::DeniedCompletion(
                candidate.admission.take().expect("actual denied admission"),
            ));
        }
        // This is admission-only uncertainty. It cannot enter ordinary Native
        // completion commit, result publication, timeout rollback or retry.
        return Ok(Received::EffectsIndeterminate);
    }
    // A verified candidate remains owned, without rerunning the provider, if
    // retained-result capacity refuses. No Native stage has occurred yet.
    if candidate.result_slot.is_none() {
        candidate.result_slot = Some(
            installed
                .storage
                .reserve_result(key)
                .map_err(ResourceCause::Storage)?,
        );
    }
    let witness = owner.request().ok_or(ResourceCause::Unavailable)?;
    if candidate.admission.is_none() {
        let identity = witness.request();
        let handle = identity.request_handle();
        let digest = identity
            .lowered()
            .resource_descriptor()
            .ok_or(ResourceCause::Unavailable)?
            .payload_contract_digest()
            .clone();
        let raw = worth_signal::facade::RawCompletionEnvelope::new(
            handle.request_id(),
            handle.generation(),
            handle.branch_epoch(),
            identity.attempt(),
            digest,
            u64::try_from(candidate.frame.length()).map_err(|_| ResourceCause::Unavailable)?,
        );
        let validated = runtime
            .bridge
            .conditional()
            .validate_owned_async_completion_envelope(witness, raw)
            .map_err(ResourceCause::Completion)?;
        let admitted = runtime
            .bridge
            .conditional()
            .admit_owned_async_completion(witness, &validated)
            .map_err(ResourceCause::Completion)?;
        candidate.admission = Some(admitted);
    }
    let admission = candidate
        .admission
        .as_ref()
        .expect("actual Native admission retained before handoff");
    if admission.report().denied_completion().is_some() {
        runtime
            .bridge
            .conditional()
            .release_process_local_completion_denial(owner, admission)
            .map_err(ResourceCause::Managed)?;
        // Preserve the actual denied admission as the stop, after its real
        // Native denial record has been released by the genuine owner.
        return Err(ResourceCause::DeniedCompletion(
            candidate.admission.take().expect("actual denied admission"),
        ));
    }
    execution.consult().map_err(ResourceCause::Request)?;
    let committed = runtime
        .bridge
        .conditional()
        .commit_owned_async_completion(witness, admission)
        .map_err(ResourceCause::Completion)?;
    // This is a genuine committed result even when timeout cleanup or the
    // later original request handoff refuses. Store all custody first.
    *completion = Some(committed);
    let (kind, value) = candidate
        .verified
        .take()
        .expect("committed typed projection");
    occurrence.result = Some(
        candidate
            .result_slot
            .take()
            .expect("R slot admitted before Native stage")
            .retain(kind, value),
    );
    execution.consult().map_err(ResourceCause::Request)?;
    Ok(Received::Committed)
}
