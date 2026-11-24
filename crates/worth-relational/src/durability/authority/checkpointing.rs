//! Fund and frame exactly one selected native checkpoint for its placement.
use crate::durability::checkpoint_selection_denial::CanonicalCheckpointAdmissionError;
use crate::durability::data::{
    DurabilityError, RelationalNativeCheckpointCaptureDenial as CaptureDenial,
};
use crate::execution::RelationalExecutionDenialCause as Cause;

pub(super) fn encode_native_capture(
    basis: super::checkpoint_capture::CapturedCheckpointBasis,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    artifact_policy: crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    selection: &crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection,
    cold_source: Option<std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>>,
    medium: Option<
        std::sync::Arc<crate::durability::published_medium::RelationalDurableMediumBinding>,
    >,
) -> Result<
    (
        crate::durability::data::RelationalNativeCheckpoint,
        crate::branch::transition::RelationalBranchTransitionSequence,
    ),
    CaptureDenial,
> {
    // One capture is one stage: every write, hash and copy charges its meter
    // cumulatively, so the request's work ceiling bounds the whole capture.
    let mut answer = None;
    let handoff = execution.run_single_stage(|stage| {
        let mut control =
            crate::durability::log::native_checkpoint_codec::CheckpointEncodingControl::Original {
                execution: *execution,
                stage,
            };
        answer = Some((|| -> Result<_, CaptureDenial> {
            let captured = basis.build_native_capture(
                execution,
                policy,
                cold_source,
                &artifact_policy,
                &mut control,
            )?;
            let mut captured = captured.seal_ordinary();
            let manifest_publication = if !artifact_policy.is_embedded() {
                artifact_policy.publication_source().map(|source| {
                    selection.prepare(
                        captured.manifest.clone(),
                        source,
                        captured.history_members.clone(),
                    )
                })
            } else {
                None
            };
            let coverage = captured.manifest.covered_position;
            let transitions = captured.manifest.coverage.up_to_branch_transition;
            let component_holds = std::mem::take(&mut captured.component_holds);
            let canonical_publication = captured.canonical_publication.take();
            let encoded = crate::durability::log::native_checkpoint_codec::encode_captured(
                captured,
                execution,
                policy,
                &artifact_policy,
                &mut control,
            )?;
            Ok((
                manifest_publication,
                coverage,
                transitions,
                component_holds,
                canonical_publication,
                encoded,
            ))
        })());
    });
    let (
        manifest_publication,
        coverage,
        transitions,
        component_holds,
        canonical_publication,
        (bytes, sections, members, publications),
    ) = match (handoff, answer) {
        (Ok(_), Some(answer)) => answer,
        (Ok(_), None) => unreachable!("a completed checkpoint stage ran its capture"),
        (Err(stop), answer) => {
            let cause = Cause::from(stop);
            // A concrete inner refusal outranks the generic nested-stop handoff.
            match answer {
                Some(Err(inner)) if cause == Cause::NestedStopped => Err(inner),
                _ => Err(CaptureDenial::Request(cause)),
            }
        }
    }?;
    crate::execution::check_request(execution)
        .map_err(crate::durability::data::RelationalNativeCheckpointCaptureDenial::Request)?;

    let checkpoint = crate::durability::data::RelationalNativeCheckpoint::from_captured_bytes(
        bytes,
        sections,
        coverage,
        transitions,
    )
    .with_component_holds(component_holds)
    .with_canonical_publication(canonical_publication)
    .with_captured_members(members)
    .with_region_publications(if manifest_publication.is_some() {
        publications
    } else {
        Vec::new()
    })
    .with_manifest_publication(manifest_publication)
    .bind_publication(medium, execution)?;
    Ok((checkpoint, transitions))
}

pub(super) fn checkpoint_admission_error(
    denial: CanonicalCheckpointAdmissionError,
) -> DurabilityError {
    match denial {
        CanonicalCheckpointAdmissionError::PublicationInFlight => {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CheckpointPublicationInFlight,
                "checkpoint denied while canonical publication is in flight",
            )
        }
        CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(
            commit_id,
        ) => DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::PerformedPublicationRequiresSettlement,
            format!(
                "checkpoint denied until performed commit {} completes owner settlement",
                commit_id.0
            ),
        ),
    }
}
