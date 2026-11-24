//! Bind each reconstructed current root before its original source becomes history.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::history_recovery_denial::RelationalHistoryRecoveryDenial;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use std::sync::Arc;

/// Reconstructive single-commit update. The catalog's existing full-payload
/// classifier alone decides whether replay-issued private evidence survives.
pub fn synchronize_recovered_source_commit(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &PositionedCanonicalCommit,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
    rebind_performed: impl FnOnce(
        &PositionedCanonicalCommit,
        Arc<worth_relational_branch::internal_port::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>,
        contracts::branch::target::RelationalBranchRootDescriptor,
    ) -> Result<(), &'static str>,
) -> Result<Arc<CanonicalCommitEnvelope>, RelationalHistoryRecoveryDenial> {
    let prepared = prepare_recovered_source_catalog(history, execution, source)?;
    let artifact = &prepared.artifact;
    let canonical = artifact
        .resident_envelope()
        .ok_or_else(|| "prepared source has no admitted canonical body".to_owned())?;
    let residency = artifact.envelope_residency();
    let roots = artifact.roots().clone();
    history.with_ledger_mut(|ledger| {
        ledger
            .commit_catalog
            .install_source_recovery(prepared.expected.as_ref(), Arc::clone(artifact))
            .map_err(RelationalHistoryRecoveryDenial::Catalog)?;
        ledger
            .commit_envelopes
            .insert(source.commit.commit_id, Arc::clone(&residency));
        ledger
            .patch_stream_index
            .insert(source.position(), source.commit.commit_id);
        Ok::<_, RelationalHistoryRecoveryDenial>(())
    })?;
    let mut cell = history
        .branch_cell(&source.branch_context)
        .ok_or_else(|| "recovered source branch is unavailable".to_owned())?;
    let Some(root) = cell.root() else {
        return Ok(canonical);
    };
    if root.commit_id() != Some(source.commit.commit_id) {
        return Ok(canonical);
    }
    rebind_performed(source, residency, roots).map_err(str::to_owned)?;
    if root.links_envelope(&canonical) {
        return Ok(canonical);
    }
    let original = original_for_envelope(history, execution, &canonical)?;
    let next = if Arc::ptr_eq(&canonical, source.canonical_arc()) {
        root.readmit_canonical_envelope(execution, source, original, symbols)
    } else {
        root.relink_canonical_envelope(execution, original, symbols)
    }
    .map_err(RelationalHistoryRecoveryDenial::Root)?;
    let retirement = history
        .reserve_branch_head_retirement(cell.identity(), &root, cell.head_retention())
        .map_err(|denial| format!("recovered source root retention denied: {denial:?}"))?;
    cell.install_root(next.clone());
    retirement.transfer_head(&next).replace_head();
    Ok(canonical)
}

pub(super) struct PreparedRecoveredSourceCatalog {
    pub(super) expected: Option<Arc<worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact>>,
    pub(super) artifact: Arc<worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact>,
}
pub(super) fn prepare_recovered_source_catalog(
    history: &HistorySubsystem,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    positioned: &PositionedCanonicalCommit,
) -> Result<PreparedRecoveredSourceCatalog, RelationalHistoryRecoveryDenial> {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as Wire;
    use worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact as Artifact;
    let expected = history.commit_artifact(positioned.commit.commit_id);
    if let Some(existing) = &expected
        && let Some(resident) = existing.resident_envelope()
        && Wire::matches_positioned_payload_for_request(positioned, &resident, request)
            .map_err(RelationalHistoryRecoveryDenial::Source)?
    {
        return Ok(PreparedRecoveredSourceCatalog {
            artifact: Arc::clone(existing),
            expected,
        });
    }
    let cell = expected
        .as_ref()
        .map(|artifact| artifact.envelope_residency())
        .or_else(|| {
            history
                .ledger
                .read()
                .commit_envelopes
                .get(&positioned.commit.commit_id)
                .cloned()
        });
    let artifact =
        if let Some((entry, source)) = cell.as_ref().and_then(|cell| cell.source_placement()) {
            let directory = cell.as_ref().and_then(|cell| cell.selected_directory());
            let artifact = Artifact::from_index(entry.clone(), source)
                .map_err(RelationalHistoryRecoveryDenial::Artifact)?;
            let residency = artifact.envelope_residency();
            residency
                .bind_recovered_positioned(request, positioned)
                .map_err(RelationalHistoryRecoveryDenial::Source)?;
            if let Some(directory) = directory {
                let _displaced = residency
                    .install_selected(entry, directory)
                    .map_err(|cause| {
                        RelationalHistoryRecoveryDenial::Invalid(format!(
                            "selected source replacement: {cause:?}"
                        ))
                    })?;
            }
            artifact.prepare_catalog_owner(request).map_err(|cause| {
                RelationalHistoryRecoveryDenial::Root(
                contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Request(
                    cause,
                ),
            )
            })?
        } else {
            let carried = cell
                .as_ref()
                .and_then(|cell| cell.original_owner())
                .or_else(|| {
                    history
                        .branch_cell(&positioned.branch_context)
                        .and_then(|cell| cell.root())
                        .filter(|root| root.commit_id() == Some(positioned.commit.commit_id))
                        .and_then(|root| root.canonical_payload())
                });
            let canonical = match carried {
                Some(original)
                    if Wire::matches_positioned_payload_for_request(
                        positioned,
                        original.envelope(),
                        request,
                    )
                    .map_err(RelationalHistoryRecoveryDenial::Source)? =>
                {
                    Arc::clone(original.envelope())
                }
                _ => Arc::clone(positioned.canonical_arc()),
            };
            prepare_catalog_artifact_for_envelope(
                history,
                request,
                canonical,
                expected.as_ref().map(|a| a.roots()),
            )?
        };
    Ok(PreparedRecoveredSourceCatalog { expected, artifact })
}

pub(super) fn prepare_catalog_artifact_for_envelope(
    history: &HistorySubsystem,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    envelope: Arc<CanonicalCommitEnvelope>,
    descriptor: Option<&contracts::branch::target::RelationalBranchRootDescriptor>,
) -> Result<
    Arc<
        worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact,
    >,
    RelationalHistoryRecoveryDenial,
> {
    use worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact as Artifact;
    let cell = history
        .ledger
        .read()
        .commit_envelopes
        .get(&envelope.commit.commit_id)
        .cloned();
    // A retained indexed source already owns its publication; no Original bytes are allocated.
    let artifact = if let Some((entry, source)) =
        cell.as_ref().and_then(|cell| cell.source_placement())
    {
        let digest = worth_relational_branch::internal_port::history::commit::payload_commitment::digest_envelope_for_request(
            entry.position,
            &envelope,
            request,
        )
        .map_err(RelationalHistoryRecoveryDenial::Source)?;
        if entry.commit != envelope.commit || entry.payload_digest != digest {
            return Err(RelationalHistoryRecoveryDenial::Artifact(
                contracts::history::commit::artifact_denial::RelationalCommitArtifactDenial::RootLinkage,
            ));
        }
        let artifact = Artifact::from_index(entry.clone(), source)
            .map_err(RelationalHistoryRecoveryDenial::Artifact)?;
        if let Some(directory) = cell.as_ref().and_then(|cell| cell.selected_directory()) {
            let _displaced = artifact
                .envelope_residency()
                .install_selected(entry, directory)
                .map_err(|cause| {
                    RelationalHistoryRecoveryDenial::Invalid(format!(
                        "selected source preservation: {cause:?}"
                    ))
                })?;
        }
        artifact
    } else {
        let root = history
            .branch_cell(&envelope.branch_context)
            .and_then(|cell| cell.root())
            .filter(|root| root.links_envelope(&envelope));
        let carried = cell
            .as_ref()
            .and_then(|cell| cell.original_owner())
            .filter(|original| Arc::ptr_eq(original.envelope(), &envelope))
            .or_else(|| root.as_ref().and_then(|root| root.canonical_payload()));
        let original = match carried {
            Some(original) => original,
            None => worth_relational_branch::internal_port::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(request, envelope)
                .map_err(RelationalHistoryRecoveryDenial::Root)?,
        };
        match (root, descriptor) {
            (Some(root), _) => {
                Artifact::from_envelope_with_root(Arc::clone(original.envelope()), root)
            }
            (None, Some(descriptor)) => {
                Artifact::from_preencoded_with_descriptor(original, descriptor.clone())
            }
            (None, None) => Artifact::from_preencoded(original),
        }
        .map_err(RelationalHistoryRecoveryDenial::Artifact)?
    };
    artifact.prepare_catalog_owner(request).map_err(|cause| {
        RelationalHistoryRecoveryDenial::Root(
            contracts::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Request(
                cause,
            ),
        )
    })
}

pub(super) fn original_for_envelope(
    history: &HistorySubsystem,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    envelope: &Arc<CanonicalCommitEnvelope>,
) -> Result<
    Arc<worth_relational_branch::internal_port::history::commit::original_payload::RelationalOriginalRootCommitEvidence>,
    RelationalHistoryRecoveryDenial,
>{
    let carried = history
        .ledger
        .read()
        .commit_envelopes
        .get(&envelope.commit.commit_id)
        .cloned()
        .and_then(|cell| cell.original_owner())
        .filter(|original| Arc::ptr_eq(original.envelope(), envelope));
    match carried {
        Some(original) => Ok(original),
        None => {
            worth_relational_branch::internal_port::history::commit::original_payload::RelationalOriginalRootCommitEvidence::prepare(
                request,
                Arc::clone(envelope),
            )
            .map_err(RelationalHistoryRecoveryDenial::Root)
        }
    }
}
