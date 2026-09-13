use super::super::*;
use super::pending::PendingProjectionBasis;

pub(super) fn seal(
    store: StableStoreIdentity,
    selection: &PhysicalSourceSelection,
    freshness: &StoreRecoveryBindingFreshnessSample,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
    selected_source: &RecoverySelectedSourceInventory,
    successor_candidate: Option<RecoveryObservedSuccessorCandidate>,
    pending: PendingProjectionBasis<'_>,
    staging: RecoveryStagingLayoutPlan,
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
        &staging,
    );
    let publication_identity = publication_identity(basis_identity);
    let candidate = if pending.projections.is_empty() {
        super::super::publication_candidate::RecoveryCandidateBasis {
            root: selection.root().selected().manifest().clone(),
            referenced_artifacts: Box::new([]),
            artifacts: Box::new([]),
            materialization_cost: CandidateMaterializationCost::default(),
            staged_current_selector: selection.root().selected().selector(),
        }
    } else {
        super::super::publication_candidate::build(
            store,
            staging.base_image(),
            selected_source,
            successor_candidate,
            selection.root().selected().selector().format(),
            publication_identity,
        )
        .map_err(|denial| match denial {
            super::super::publication_candidate::CandidateBuildDenial::SuccessorCandidate(
                denial,
            ) => ExecutionBasisDenial::SuccessorCandidate(denial),
            super::super::publication_candidate::CandidateBuildDenial::Invalid => {
                ExecutionBasisDenial::Invalid
            }
        })?
    };
    let plan_identity = super::super::identity::bind_publication_candidates(
        basis_identity,
        &candidate.root,
        selection.root().selected().selector().format(),
        &candidate.referenced_artifacts,
        &candidate.artifacts,
    );
    let actions = publication_actions(&candidate.artifacts);
    let staging_commands = (staging.commands.len() as u64)
        .checked_mul(2)
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let (current_selector, root_protocol_counters) =
        super::selector_closeout::select_staged_current(
            &candidate,
            store,
            selection.root().selected().selector().format(),
        )?;
    let materialization_cost = candidate.materialization_cost;
    let created_artifacts = created_artifacts(&staging, &candidate.artifacts);
    let publication = RecoveryPublicationPlan {
        store,
        checkpoint: pending.checkpoint,
        source_generation: pending.source_generation,
        staging_generation: pending.staging_generation,
        actions: actions.clone().into_boxed_slice(),
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
    };
    let quiescence = RecoveryQuiescencePlan {
        staging_commands,
        publication_commands: actions.len() as u64,
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

fn created_artifacts(
    staging: &RecoveryStagingLayoutPlan,
    candidates: &[RecoveryPublicationCandidateArtifact],
) -> Box<[RecordArtifactFile]> {
    let mut artifacts = BTreeSet::new();
    artifacts.extend(staging.commands().iter().map(|command| command.artifact()));
    artifacts.extend(candidates.iter().map(|candidate| candidate.artifact()));
    artifacts.into_iter().collect::<Vec<_>>().into_boxed_slice()
}

fn publication_actions(
    candidates: &[RecoveryPublicationCandidateArtifact],
) -> Vec<RecoveryPublicationAction> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let mut actions = Vec::with_capacity(candidates.len() * 2 + 2);
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
    actions
}

fn publication_identity(plan_identity: [u8; 32]) -> u64 {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&plan_identity[..8]);
    u64::from_le_bytes(bytes).max(1)
}
