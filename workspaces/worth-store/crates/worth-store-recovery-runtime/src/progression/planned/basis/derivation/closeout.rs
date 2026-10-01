use super::super::*;
use super::pending::PendingProjectionBasis;
use sha2::Digest;

pub(super) fn seal(
    store: StableStoreIdentity,
    selection: &PhysicalSourceSelection,
    freshness: &StoreRecoveryBindingFreshnessSample,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
    historical_consumed: &HistoricalConsumedOperationSet,
    selected_source: &RecoverySelectedSourceInventory,
    verified_drops: &[worth_store_physical_format::PersistedRecordIdentity],
    validated_manifest_cleanup: Option<crate::orchestration::ValidatedManifestResidueCleanup>,
    successor_candidate: Option<RecoveryObservedSuccessorCandidate>,
    pending: PendingProjectionBasis<'_>,
    staging: RecoveryStagingLayoutPlan,
    maximum_manifest_entries: u64,
    maximum_staging_bytes: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<
    (
        RecoveryStagingLayoutPlan,
        RecoveryPublicationPlan,
        RecoveryQuiescencePlan,
        CandidateMaterializationCost,
        crate::entry::PhysicalRecoveryRootProtocolCounters,
    ),
    ExecutionBasisDenial,
> {
    let basis_identity = super::super::identity::plan_identity(
        store,
        pending.checkpoint,
        selection,
        freshness,
        fates,
        redo,
        historical_consumed,
        &staging,
        validated_manifest_cleanup,
        allowance,
    )?;
    let publication_identity = validated_manifest_cleanup.map_or_else(
        || publication_identity(basis_identity),
        |cleanup| cleanup.intent().publication(),
    );
    let candidate = if pending.projections.is_empty()
        && pending.source_copies.is_empty()
        && validated_manifest_cleanup.is_none()
    {
        super::super::publication_candidate::RecoveryCandidateBasis {
            root: selection.root().selected().manifest().clone(),
            referenced_artifacts: Box::new([]),
            artifacts: Box::new([]),
            materialization_cost: CandidateMaterializationCost::default(),
            staged_current_selector: selection.root().selected().selector(),
            release_topology: None,
        }
    } else {
        super::super::publication_candidate::build(
            store,
            staging.base_image(),
            selected_source,
            selection.page_facts().placements(),
            successor_candidate,
            selection.root().selected().selector().format(),
            publication_identity,
            selection
                .root()
                .selected()
                .manifest()
                .requires_maintenance_protocol()
                || !verified_drops.is_empty()
                || !redo.rewrites().is_empty()
                || !redo.source_copies().is_empty()
                || validated_manifest_cleanup.is_some(),
            verified_drops,
            maximum_manifest_entries,
            maximum_staging_bytes,
            allowance,
        )
        .map_err(candidate_denial)?
    };
    if let Some(cleanup) = validated_manifest_cleanup {
        if !candidate_root_matches_cleanup(
            &candidate.root,
            selection.root().selected().selector().format(),
            cleanup.intent().candidate_root_generation(),
            cleanup.intent().candidate_root_sha256(),
            allowance,
        )? {
            return Err(ExecutionBasisDenial::Invalid);
        }
    }
    let plan_identity = super::super::identity::bind_publication_candidates(
        basis_identity,
        &candidate.root,
        selection.root().selected().selector().format(),
        &candidate.referenced_artifacts,
        &candidate.artifacts,
        allowance,
    )?;
    let actions = publication_actions(&candidate.artifacts, allowance)?;
    let publication_commands = actions.len() as u64;
    let staging_commands = staging.scheduler_command_count();
    let (current_selector, root_protocol_counters) =
        super::selector_closeout::select_staged_current(
            &candidate,
            store,
            selection.root().selected().selector().format(),
        )?;
    let materialization_cost = candidate.materialization_cost;
    let created_artifacts = created_artifacts(&staging, &candidate.artifacts, allowance)?;
    let discarded_pending_bytes = PlanningResidentAllowance::vector_bytes(&pending.projections)?
        .checked_add(PlanningResidentAllowance::vector_bytes(
            &pending.source_copies,
        )?)
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let checkpoint = pending.checkpoint;
    let source_generation = pending.source_generation;
    let staging_generation = pending.staging_generation;
    drop(pending);
    allowance.release(discarded_pending_bytes);
    let publication = RecoveryPublicationPlan {
        store,
        checkpoint,
        source_generation,
        staging_generation,
        actions,
        plan_identity,
        root_protocol: worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan::from_catalog_candidate(
            RecordArtifactFile::CatalogCandidate {
                publication: publication_identity,
            },
        )
        .expect("the Phase 4 publication identity always names a catalog candidate"),
        current_selector,
        recovered_root: candidate.root,
        referenced_artifacts: candidate.referenced_artifacts,
        candidates: candidate.artifacts,
        created_artifacts,
        release_topology: candidate.release_topology,
    };
    let quiescence = RecoveryQuiescencePlan {
        staging_commands,
        publication_commands,
        expected_live_commands_after_close: 0,
        expected_live_media_handles_after_close: 0,
    };
    Ok((
        staging,
        publication,
        quiescence,
        materialization_cost,
        root_protocol_counters,
    ))
}

fn candidate_root_matches_cleanup(
    root: &DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    generation: u64,
    sha256: [u8; 32],
    allowance: &mut PlanningResidentAllowance,
) -> Result<bool, ExecutionBasisDenial> {
    if root.generation() != generation {
        return Ok(false);
    }
    let bytes =
        super::super::publication_candidate::encoding::root_manifest(root, format, allowance)
            .map_err(candidate_denial)?;
    let matches = <[u8; 32]>::from(sha2::Sha256::digest(&bytes)) == sha256;
    let backing = PlanningResidentAllowance::vector_bytes(&bytes)?;
    drop(bytes);
    allowance.release(backing);
    Ok(matches)
}

pub(in crate::progression::planned::basis) fn candidate_denial(
    denial: super::super::publication_candidate::CandidateBuildDenial,
) -> ExecutionBasisDenial {
    use super::super::publication_candidate::CandidateBuildDenial;
    match denial {
        CandidateBuildDenial::Memory(PlanningMemoryDenial::RecoveryMemoryBytes { observed }) => {
            ExecutionBasisDenial::RecoveryMemoryBytes { observed }
        }
        CandidateBuildDenial::Memory(PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        }) => ExecutionBasisDenial::PublicationCandidateAllocation {
            requested_bytes,
            cause,
        },
        CandidateBuildDenial::StagingBytes { observed } => {
            ExecutionBasisDenial::StagingBytes { observed }
        }
        CandidateBuildDenial::SuccessorCandidate(denial) => {
            ExecutionBasisDenial::SuccessorCandidate(denial)
        }
        CandidateBuildDenial::Invalid => ExecutionBasisDenial::Invalid,
    }
}

#[cfg(test)]
mod manifest_cleanup_tests {
    use super::*;
    use worth_store_physical_format::PhysicalRecordFormatDeclaration;

    #[test]
    fn root_only_cleanup_requires_exact_candidate_generation_and_digest() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let root = DurablePhysicalRootManifest::builder(6, 7, 4, 19)
            .admit()
            .unwrap();
        let digest = sha2::Sha256::digest(root.encode(format)).into();
        let mut allowance = PlanningResidentAllowance::new(0, 4096).unwrap();
        assert!(candidate_root_matches_cleanup(&root, format, 6, digest, &mut allowance).unwrap());
        assert!(!candidate_root_matches_cleanup(&root, format, 7, digest, &mut allowance).unwrap());
        assert!(
            !candidate_root_matches_cleanup(&root, format, 6, [8; 32], &mut allowance).unwrap()
        );
        assert_eq!(allowance.used(), 0);
    }
}

fn created_artifacts(
    staging: &RecoveryStagingLayoutPlan,
    candidates: &[RecoveryPublicationCandidateArtifact],
    allowance: &mut PlanningResidentAllowance,
) -> Result<Box<[RecordArtifactFile]>, ExecutionBasisDenial> {
    let count = staging
        .commands()
        .len()
        .checked_add(staging.source_copies().len())
        .and_then(|count| count.checked_add(candidates.len()))
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let mut artifacts = allowance
        .reserve(count)
        .map_err(|denial| candidate_denial(denial.into()))?;
    artifacts.extend(staging.commands().iter().map(|command| command.artifact()));
    artifacts.extend(
        staging
            .source_copies()
            .iter()
            .map(|copy| RecordArtifactFile::ExtentArena {
                arena: copy.intent().destination().arena_range().arena().get(),
            }),
    );
    artifacts.extend(candidates.iter().map(|candidate| candidate.artifact()));
    artifacts.sort_unstable();
    artifacts.dedup();
    allowance
        .into_box(artifacts)
        .map_err(|denial| candidate_denial(denial.into()))
}

fn publication_actions(
    candidates: &[RecoveryPublicationCandidateArtifact],
    allowance: &mut PlanningResidentAllowance,
) -> Result<Box<[RecoveryPublicationAction]>, ExecutionBasisDenial> {
    if candidates.is_empty() {
        return Ok(Box::new([]));
    }
    let count = candidates
        .len()
        .checked_mul(2)
        .and_then(|count| count.checked_add(2))
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let mut actions = allowance
        .reserve(count)
        .map_err(|denial| candidate_denial(denial.into()))?;
    for candidate in candidates {
        actions.push(RecoveryPublicationAction::MaterializeRootCandidate {
            artifact: candidate.artifact(),
        });
        actions.push(RecoveryPublicationAction::SynchronizeRootCandidate {
            artifact: candidate.artifact(),
        });
    }
    actions.push(RecoveryPublicationAction::ReplaceRootProtocol);
    actions.push(RecoveryPublicationAction::SynchronizeStoreNamespace);
    allowance
        .into_box(actions)
        .map_err(|denial| candidate_denial(denial.into()))
}

fn publication_identity(plan_identity: [u8; 32]) -> u64 {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&plan_identity[..8]);
    u64::from_le_bytes(bytes).max(1)
}

#[cfg(test)]
#[path = "closeout/budget_tests.rs"]
mod budget_tests;
