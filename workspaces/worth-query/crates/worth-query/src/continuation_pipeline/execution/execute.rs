use crate::application::{
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput, WorthQueryDomainEntryMarker,
    WorthQueryDomainOperatingContext, WorthQueryInstalledDomainDeclarationContext,
};
use crate::binding_pipeline::{WorthQueryBindingRequestDescriptor, WorthQueryBindingWitnessCheck};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::installed_authority::execution_installed_authority_drift_transcript;
use super::readmission::{
    current_readmission_evidence_from_handle, validate_execution_readmission,
};
use super::readmission_transcript::transcript_from_readmission_denial;
use super::support::{linked_artifacts_identity, linked_from_prepared, transcript_digest};
use super::WorthQueryContinuationExecution;
use crate::continuation_pipeline::request::WorthQueryExecutePreparedContinuationRequest;
use crate::continuation_pipeline::transcript::WorthQueryContinuationExecutionTranscript;
use crate::continuation_pipeline::WorthQueryContinuationExecutionOutcome;

fn missing_required_capability<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    prepared: &super::WorthQueryPreparedContinuation<D, I>,
) -> Option<crate::application::WorthQueryCapabilityFamily> {
    prepared
        .required_capability_families()
        .iter()
        .copied()
        .find(|family| {
            handle.installed_capability_status(*family)
                != Some(crate::application::WorthQueryCapabilityStatus::Admitted)
        })
}

pub(crate) fn execute_prepared_continuation_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &WorthQueryInstalledDomainDeclarationContext<D, C>,
    request: WorthQueryExecutePreparedContinuationRequest<D, I>,
) -> Result<WorthQueryContinuationExecutionTranscript<D, I>, IdentityDerivationDenial> {
    let prepared = request.into_prepared();
    let request_descriptor = WorthQueryBindingRequestDescriptor::new(
        I::Family::semantic_family_key(),
        "execute_prepared_continuation",
        prepared.bridge_routing().aspect_contract().clone(),
    );
    let linked = linked_from_prepared(&prepared)?;

    if let Err(drift) =
        crate::domain_installation::WorthQueryInstalledDomainExecutionDrift::validate_retained_for_current(
            prepared.installed_authority(),
            handle.installed_authority(),
        )
    {
        return Ok(execution_installed_authority_drift_transcript(
            request_descriptor,
            linked,
            drift,
        )?);
    }

    if prepared.operating_context_identity_digest() != handle.operating_context_identity_digest() {
        return Ok(WorthQueryContinuationExecutionTranscript::new(
            request_descriptor,
            WorthQueryContinuationExecutionOutcome::WrongWorld(
                "the prepared continuation belongs to a different admitted world".to_string(),
            ),
            vec![WorthQueryBindingWitnessCheck::failed(
                "world_alignment",
                "prepared continuation operating context digest did not match",
            )],
            transcript_digest(
                "execute_prepared_continuation",
                I::Family::semantic_family_key(),
                &linked,
                "wrong_world",
            )?,
            linked,
        ));
    }

    if let Some(family) = missing_required_capability(handle, &prepared) {
        return Ok(WorthQueryContinuationExecutionTranscript::new(
            request_descriptor,
            WorthQueryContinuationExecutionOutcome::Unsupported(format!(
                "the current admitted handle no longer admits required continuation capability {}",
                family.as_str()
            )),
            vec![WorthQueryBindingWitnessCheck::failed(
                "execution_support",
                "required continuation capability is not currently admitted",
            )],
            transcript_digest(
                "execute_prepared_continuation",
                I::Family::semantic_family_key(),
                &linked,
                "unsupported",
            )?,
            linked,
        ));
    }

    let current_evidence = current_readmission_evidence_from_handle(handle, &prepared)?;
    if let Err(denial) = validate_execution_readmission(&prepared, &current_evidence) {
        return Ok(transcript_from_readmission_denial(
            request_descriptor,
            linked,
            denial,
        )?);
    }

    if prepared.handle_identity_digest() != handle.handle_identity_digest() {
        return Ok(WorthQueryContinuationExecutionTranscript::new(
            request_descriptor,
            WorthQueryContinuationExecutionOutcome::WrongHandle(
                "the prepared continuation belongs to a different admitted handle".to_string(),
            ),
            vec![WorthQueryBindingWitnessCheck::failed(
                "handle_alignment",
                "prepared continuation handle digest did not match",
            )],
            transcript_digest(
                "execute_prepared_continuation",
                I::Family::semantic_family_key(),
                &linked,
                "wrong_handle",
            )?,
            linked,
        ));
    }

    let execution_digest =
        worth_query_evidence_identity(WorthQueryEvidenceScope::ContinuationExecutionDigest)
            .field_value(
                WorthQueryEvidenceTag::new("prepared"),
                prepared.prepared_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                prepared.family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("truth_context"),
                prepared.truth_context().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_posture"),
                prepared.basis_posture().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("workspace_contract"),
                prepared.workspace_contract().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("runtime_contract"),
                prepared.runtime_contract().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("execution_mode"),
                prepared.execution_mode().as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("required_basis_families"),
                prepared
                    .required_basis_families()
                    .iter()
                    .map(|family| family.as_str()),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("required_capability_families"),
                prepared
                    .required_capability_families()
                    .iter()
                    .map(|family| family.as_str()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("signal_posture"),
                prepared.signal_posture().as_str(),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("signal_execution_family"),
                prepared
                    .signal_execution_family()
                    .map(|family| family.as_str()),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("signal_compatibility"),
                prepared.signal_compatibility_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("linked_artifacts"),
                linked_artifacts_identity(&linked)?,
            )
            .seal()?
            .as_str()
            .to_string();
    let binding_surface = prepared.bridge_routing().binding().surface().to_string();
    let signal_execution_family = prepared.signal_execution_family();
    let execution = WorthQueryContinuationExecution::new(
        prepared,
        signal_execution_family,
        binding_surface,
        execution_digest.clone(),
    );

    Ok(WorthQueryContinuationExecutionTranscript::new(
        request_descriptor,
        WorthQueryContinuationExecutionOutcome::Executed(execution),
        vec![
            WorthQueryBindingWitnessCheck::passed("world_alignment"),
            WorthQueryBindingWitnessCheck::passed("execution_support"),
            WorthQueryBindingWitnessCheck::passed("handle_alignment"),
            WorthQueryBindingWitnessCheck::passed("prepared_continuation"),
        ],
        execution_digest,
        linked,
    ))
}
