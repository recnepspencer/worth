//! Fund and frame exactly one selected native checkpoint for its placement.
use crate::durability::checkpoint_selection_denial::CanonicalCheckpointAdmissionError;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as CaptureDenial;
use contracts::durability::data::recovery_errors::DurabilityError;
use contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use worth_relational_contracts::internal_port as contracts;

pub fn encode_native_capture(
    basis: super::checkpoint_capture::CapturedCheckpointBasis,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    artifact_policy: crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    selection: &crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection,
    cold_source: Option<std::sync::Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>>,
    medium: Option<
        std::sync::Arc<crate::durability::published_medium::RelationalDurableMediumBinding>,
    >,
) -> Result<
    (
        crate::durability::data::RelationalNativeCheckpoint,
        contracts::branch::transition::sequence::RelationalBranchTransitionSequence,
    ),
    CaptureDenial,
> {
    // One capture is one stage: every write, hash and copy charges its meter
    // cumulatively, so the request's work ceiling bounds the whole capture.
    let mut answer = None;
    let handoff = execution.run_single_stage(|stage| {
        let mut control =
            crate::durability::log::native_checkpoint_codec::CheckpointEncodingControl {
                execution: *execution,
                stage,
            };
        answer = Some((|| -> Result<_, CaptureDenial> {
            let naming_generations = basis
                .lineage_naming
                .as_ref()
                .map(|frozen| frozen.branch_generations().clone())
                .unwrap_or_default();
            let captured = basis.build_native_capture(
                execution,
                policy,
                cold_source,
                &artifact_policy,
                &mut control,
            )?;
            let mut captured = captured.seal_ordinary();
            let manifest_publication = artifact_policy.publication_source().map(|source| {
                selection
                    .prepare(
                        captured.manifest.clone(),
                        source,
                        captured.history_members.clone(),
                    )
                    .with_naming_generations(naming_generations)
            });
            let coverage = captured.manifest.covered_position;
            let transitions = captured.manifest.coverage.up_to_branch_transition;
            let component_holds = std::mem::take(&mut captured.component_holds);
            let canonical_publication = captured.canonical_publication.take();
            let encoded = crate::durability::log::native_checkpoint_codec::encode_captured(
                captured,
                execution,
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
    contracts::execution::request_custody::check_request(execution)
        .map_err(contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::Request)?;

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

pub fn checkpoint_admission_error(denial: CanonicalCheckpointAdmissionError) -> DurabilityError {
    match denial {
        CanonicalCheckpointAdmissionError::PublicationInFlight => {
            DurabilityError::new(
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CheckpointPublicationInFlight,
                "checkpoint denied while canonical publication is in flight",
            )
        }
        CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(
            commit_id,
        ) => DurabilityError::new(
            contracts::durability::data::recovery_errors::RecoveryFailureClass::PerformedPublicationRequiresSettlement,
            format!(
                "checkpoint denied until performed commit {} completes owner settlement",
                commit_id.0
            ),
        ),
    }
}
