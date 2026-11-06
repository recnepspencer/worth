use crate::application::{
    WorthQueryDeclarationBridgeRoutingChecked, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::binding_pipeline::{
    WorthQueryBindingNarrowingDecision, WorthQueryBindingRequestDescriptor,
    WorthQueryBindingTranscript, WorthQueryBindingWitnessCheck, WorthQueryContinuationBindingInput,
    WorthQueryContinuationBindingRequest, WorthQueryResolveContinuationFromTargetRequest,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::installed_authority::prepared_installed_authority_drift_transcript;
use super::outcome::{
    prepared_outcome_from_binding_outcome, prepared_outcome_from_bridge_denial_cause,
    prepared_outcome_from_signal_truth, prepared_outcome_token,
};
use super::readmission::prepared_execution_readmission_from_routing;
use super::support::{
    bridge_subject_and_signal_truth, linked_from_subject, prepared_digest,
    required_capability_families_for_prepared, signal_subject_from_bridge_subject,
    transcript_digest,
};
use crate::continuation_pipeline::artifacts::{
    basis_posture_for_families, family_for_mode, runtime_contract_for_mode, truth_context_for_mode,
    workspace_contract_for_mode, WorthQueryPreparedContinuationExecutionMode,
    WorthQueryPreparedContinuationSignalPosture,
};
use crate::continuation_pipeline::request::WorthQueryPreparedContinuationRequest;
use crate::continuation_pipeline::transcript::WorthQueryPreparedContinuationTranscript;
use crate::continuation_pipeline::{
    WorthQueryPreparedContinuation, WorthQueryPreparedContinuationOutcome,
};

pub(crate) fn prepare_continuation_from_target_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    request: WorthQueryResolveContinuationFromTargetRequest<D, I>,
) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial> {
    if let Err(drift) = handle.validate_current_installation() {
        return Ok(prepared_installed_authority_drift_transcript(
            WorthQueryBindingRequestDescriptor::new(
                I::Family::semantic_family_key(),
                "prepared_continuation",
                request.required_aspect_contract().clone(),
            ),
            crate::binding_pipeline::WorthQueryBindingLinkedArtifacts::new(),
            drift,
        )?);
    }
    let binding = handle.bind_continuation_from_target_proof(request);
    Ok(prepare_from_binding_transcript(handle, binding)?)
}

pub(crate) fn prepare_continuation_from_context_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    request: WorthQueryContinuationBindingRequest<D, I>,
) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial> {
    if let Err(drift) = handle.validate_current_installation() {
        return Ok(prepared_installed_authority_drift_transcript(
            WorthQueryBindingRequestDescriptor::new(
                I::Family::semantic_family_key(),
                "prepared_continuation",
                request.required_aspect_contract().clone(),
            ),
            crate::binding_pipeline::WorthQueryBindingLinkedArtifacts::new(),
            drift,
        )?);
    }
    let binding = handle.bind_continuation_request_from_context_proof(request);
    Ok(prepare_from_binding_transcript(handle, binding)?)
}

pub(crate) fn prepare_continuation_from_signal_checked_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    checked: crate::application::WorthQueryDeclarationSignalCompatibilityChecked<D, I>,
    bridge_request: crate::application::WorthQueryDeclarationBridgeContinuationRequest,
    required_contract: crate::application::WorthQueryDeclarationAspectContract,
) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial> {
    if let Err(drift) = handle.validate_current_installation() {
        return Ok(prepared_installed_authority_drift_transcript(
            WorthQueryBindingRequestDescriptor::new(
                I::Family::semantic_family_key(),
                "prepared_continuation",
                required_contract.clone(),
            ),
            crate::binding_pipeline::WorthQueryBindingLinkedArtifacts::new(),
            drift,
        )?);
    }
    let request = WorthQueryBindingRequestDescriptor::new(
        I::Family::semantic_family_key(),
        "prepared_continuation",
        required_contract.clone(),
    );
    let (signal_truth, subject) = bridge_subject_and_signal_truth(checked);
    let linked = linked_from_subject(&subject)?;
    Ok(prepare_from_resolved_signal_truth(
        handle,
        request,
        linked,
        signal_truth,
        subject,
        bridge_request,
        required_contract,
    )?)
}

fn prepare_continuation_from_input_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    request: WorthQueryPreparedContinuationRequest<D, I>,
) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial> {
    let (input, request_contract) = request.into_parts();
    let (bridge_request, subject) = input.into_bridge_parts();
    let required_contract = request_contract.unwrap_or_else(|| I::Family::aspect_contract());
    let request_descriptor = WorthQueryBindingRequestDescriptor::new(
        I::Family::semantic_family_key(),
        "prepared_continuation",
        required_contract.clone(),
    );
    let signal_checked =
        handle.signal_compatibility_checked(signal_subject_from_bridge_subject(subject));
    let (signal_truth, subject) = bridge_subject_and_signal_truth(signal_checked);
    let linked = linked_from_subject(&subject)?;
    Ok(prepare_from_resolved_signal_truth(
        handle,
        request_descriptor,
        linked,
        signal_truth,
        subject,
        bridge_request,
        required_contract,
    )?)
}

#[allow(clippy::too_many_arguments)]
fn prepare_from_resolved_signal_truth<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    request_descriptor: WorthQueryBindingRequestDescriptor,
    linked: crate::binding_pipeline::WorthQueryBindingLinkedArtifacts,
    signal_truth: super::support::ResolvedSignalContinuationTruth,
    subject: crate::application::WorthQueryDeclarationBridgeRoutingInput<D, I>,
    bridge_request: crate::application::WorthQueryDeclarationBridgeContinuationRequest,
    required_contract: crate::application::WorthQueryDeclarationAspectContract,
) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial> {
    if let Err(drift) = handle.validate_current_installation() {
        return Ok(prepared_installed_authority_drift_transcript(
            request_descriptor,
            linked,
            drift,
        )?);
    }
    let mut witness_checks = vec![
        WorthQueryBindingWitnessCheck::passed("continuation_binding"),
        WorthQueryBindingWitnessCheck::passed("signal_compatibility"),
    ];
    let mut narrowing = vec![WorthQueryBindingNarrowingDecision::new(
        "prepared continuation reuses retained continuation binding and lower-authority continuation truth",
    )];
    if !matches!(
        signal_truth.posture,
        WorthQueryPreparedContinuationSignalPosture::Compatible
    ) {
        witness_checks[1] =
            WorthQueryBindingWitnessCheck::failed("signal_compatibility", signal_truth.reason);
        narrowing.push(WorthQueryBindingNarrowingDecision::new(
            "prepared continuation stops before bridge preparation when retained signal compatibility is not compatible",
        ));
        let outcome = prepared_outcome_from_signal_truth(&signal_truth)
            .expect("non-compatible signal posture must stop before preparation");
        let digest = transcript_digest(
            "prepared_continuation",
            I::Family::semantic_family_key(),
            &linked,
            prepared_outcome_token(&outcome),
        )?;
        return Ok(WorthQueryPreparedContinuationTranscript::new(
            request_descriptor,
            outcome,
            witness_checks,
            narrowing,
            digest,
            linked,
        ));
    }
    let bridge_checked = handle.route_bridge_continuation_checked(subject)?;
    let outcome = prepared_outcome_from_bridge_checked(
        handle,
        &bridge_request,
        &required_contract,
        &linked,
        &signal_truth,
        &mut witness_checks,
        bridge_checked,
    );
    let digest = transcript_digest(
        "prepared_continuation",
        I::Family::semantic_family_key(),
        &linked,
        prepared_outcome_token(&outcome),
    )?;
    Ok(WorthQueryPreparedContinuationTranscript::new(
        request_descriptor,
        outcome,
        witness_checks,
        narrowing,
        digest,
        linked,
    ))
}

#[allow(clippy::too_many_arguments)]
fn prepared_outcome_from_bridge_checked<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    bridge_request: &crate::application::WorthQueryDeclarationBridgeContinuationRequest,
    required_contract: &crate::application::WorthQueryDeclarationAspectContract,
    linked: &crate::binding_pipeline::WorthQueryBindingLinkedArtifacts,
    signal_truth: &super::support::ResolvedSignalContinuationTruth,
    witness_checks: &mut Vec<WorthQueryBindingWitnessCheck>,
    bridge_checked: WorthQueryDeclarationBridgeRoutingChecked<D, I>,
) -> Result<WorthQueryPreparedContinuationOutcome<D, I>, IdentityDerivationDenial> {
    Ok(match bridge_checked {
        WorthQueryDeclarationBridgeRoutingChecked::Routed(routing) => {
            witness_checks.push(WorthQueryBindingWitnessCheck::passed("bridge_routing"));
            let required_capability_families = required_capability_families_for_prepared::<D, I>(
                bridge_request,
                signal_truth.execution_family,
            );
            let execution_readmission = prepared_execution_readmission_from_routing(
                bridge_request,
                &routing,
                required_capability_families,
            );
            let prepared_digest = prepared_digest::<D, C, I>(
                handle,
                bridge_request,
                required_contract,
                linked,
                signal_truth,
                &routing,
            )?;
            let prepared = WorthQueryPreparedContinuation::new(
                family_for_mode(bridge_request.mode()),
                truth_context_for_mode(routing.truth_context()),
                basis_posture_for_families(&signal_truth.basis_families),
                workspace_contract_for_mode(bridge_request.mode()),
                runtime_contract_for_mode(bridge_request.mode()),
                WorthQueryPreparedContinuationExecutionMode::ExplicitBridgeLowering,
                signal_truth.basis_families.clone(),
                execution_readmission,
                routing,
                signal_truth.posture,
                signal_truth.execution_family,
                signal_truth.digest.clone(),
                prepared_digest,
            );
            WorthQueryPreparedContinuationOutcome::Prepared(prepared)
        }
        WorthQueryDeclarationBridgeRoutingChecked::Deferred(deferred) => {
            witness_checks.push(WorthQueryBindingWitnessCheck::failed(
                "bridge_routing",
                deferred.reason(),
            ));
            WorthQueryPreparedContinuationOutcome::Deferred(deferred.reason().to_string())
        }
        WorthQueryDeclarationBridgeRoutingChecked::Denied(denied) => {
            witness_checks.push(WorthQueryBindingWitnessCheck::failed(
                "bridge_routing",
                denied.reason(),
            ));
            prepared_outcome_from_bridge_denial_cause(denied.cause(), denied.reason())
        }
        WorthQueryDeclarationBridgeRoutingChecked::Failed(failed) => {
            witness_checks.push(WorthQueryBindingWitnessCheck::failed(
                "bridge_routing",
                failed.reason(),
            ));
            WorthQueryPreparedContinuationOutcome::Failed(failed.reason().to_string())
        }
    })
}

fn prepare_from_binding_transcript<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    binding: WorthQueryBindingTranscript<WorthQueryContinuationBindingInput<D, I>>,
) -> Result<WorthQueryPreparedContinuationTranscript<D, I>, IdentityDerivationDenial> {
    let required_contract = binding.request().required_aspect_contract().clone();
    let checked = binding.into_checked();
    let linked = checked.linked_artifacts().clone();
    let request = WorthQueryBindingRequestDescriptor::new(
        I::Family::semantic_family_key(),
        "prepared_continuation",
        required_contract.clone(),
    );

    Ok(match checked.into_parts() {
        (crate::binding_pipeline::WorthQueryBindingOutcome::Bound(input), _, _) => {
            prepare_continuation_from_input_on_handle(
                handle,
                WorthQueryPreparedContinuationRequest::new(input)
                    .with_required_aspect_contract(required_contract),
            )?
        }
        (outcome, digest, _) => {
            let (prepared_outcome, witness_name, narrowing) =
                prepared_outcome_from_binding_outcome(outcome);
            WorthQueryPreparedContinuationTranscript::new(
                request,
                prepared_outcome,
                vec![WorthQueryBindingWitnessCheck::failed(
                    witness_name,
                    narrowing.reason(),
                )],
                vec![narrowing],
                digest,
                linked,
            )
        }
    })
}
