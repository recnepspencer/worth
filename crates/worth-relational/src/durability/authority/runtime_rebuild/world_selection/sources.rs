//! Join World descriptions to original Native artifacts or admitted image rows.
use super::*;
use crate::branch::{RecordedRelationalComponentTarget as Target, RelationalRootCommitEvidence};
use crate::history::{data::CommitId, RetainedCanonicalIndex};

impl WorldSelectionReplay {
    pub(super) fn entry(
        &self,
        commit: CommitId,
        runtime: &RelationalRuntime,
        plan: &RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetainedCanonicalIndex, Stop> {
        let artifact = runtime
            .history
            .commit_artifact(commit)
            .ok_or_else(|| corrupt("World component has no Native artifact"))?;
        if let Some(index) = self.tails.get(&commit) {
            let original = &plan.tail_log[*index];
            let (digest, _) = original
                .journal_placement()
                .ok_or_else(|| corrupt("World component has no original Journal placement"))?;
            let positioned = artifact
                .envelope_residency()
                .positioned_if_resident()
                .ok_or_else(|| corrupt("World component has no replayed original body"))?;
            if !crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::matches_positioned_payload_for_request(
                &positioned, original.positioned().canonical_arc(), request,
            )? || positioned.position() != original.position() {
                return Err(corrupt("World component differs from its admitted original Native body"));
            }
            let observed = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::positioned_payload_digest(&positioned)
                .map_err(|_| corrupt("World component original payload could not be committed"))?;
            if observed != digest {
                return Err(corrupt("World component original payload digest mismatch"));
            }
            RetainedCanonicalIndex::from_publication(&positioned, artifact.roots(), digest, request)
                .map_err(|cause| {
                    crate::durability::data::RelationalNativeCheckpointCaptureDenial::from(cause)
                        .into()
                })
        } else {
            let (entry, source) = artifact
                .envelope_residency()
                .source_placement()
                .ok_or_else(|| {
                    corrupt("World component has no selected Native checkpoint source")
                })?;
            let admitted = runtime
                .durability
                .checkpoint_selection()
                .admitted_commit(&entry)?
                .ok_or_else(|| {
                    corrupt("World component lacks an admitted Native checkpoint row")
                })?;
            if !runtime
                .durability
                .checkpoint_selection()
                .matches_admitted_source(&admitted, &source)
            {
                return Err(corrupt(
                    "World component names a foreign Native checkpoint source",
                ));
            }
            Ok(entry)
        }
    }
    pub(super) fn recorded_source(
        &self,
        source: &Source,
        runtime: &RelationalRuntime,
        plan: &RecoveryPlan,
        roots: &RecoveredRootInventory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Recorded, Stop> {
        match source {
            Source::Recorded(recorded) => Ok(recorded.clone()),
            Source::Published(coordinates) => {
                let entry = self.entry(coordinates.commit(), runtime, plan, request)?;
                if coordinates.position() != entry.position
                    || coordinates.digest().bytes() != &entry.payload_digest
                {
                    return Err(corrupt(
                        "World occurrence Native source coordinates mismatch",
                    ));
                }
                let observations = roots
                    .publication_observations(coordinates.commit())
                    .ok_or_else(|| corrupt("World occurrence Native source has no observation"))?;
                let mut selected = None;
                for descriptor in observations {
                    if descriptor.branch_id() != &entry.commit.branch_id
                        || descriptor.posture() != crate::branch::RelationalBranchBasisPosture::Live
                    {
                        continue;
                    }
                    let worth_foundational::FoundationalBranchTarget::Basis(target) =
                        descriptor.reference().target()
                    else {
                        continue;
                    };
                    if target.selected_commit_id() != entry.commit.commit_id.0
                        || target.version_id() != entry.commit.version_id.0
                        || target.roots() != &entry.root
                        || target.parent_commit_ids().iter().copied().ne(entry
                            .commit
                            .parents
                            .iter()
                            .map(|id| id.0))
                    {
                        continue;
                    }
                    let candidate = Recorded::from_readmitted_source(request, descriptor, &entry)
                        .map_err(description)?;
                    if selected
                        .as_ref()
                        .is_some_and(|previous| previous != &candidate)
                    {
                        return Err(corrupt("World occurrence Native source is ambiguous"));
                    }
                    selected = Some(candidate);
                }
                selected
                    .ok_or_else(|| corrupt("World occurrence has no matching actual Native source"))
            }
        }
    }
    pub(super) fn require_source(
        &self,
        recorded: &Recorded,
        root: &Arc<RelationalBranchRoot>,
        runtime: &RelationalRuntime,
        plan: &RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Stop> {
        match recorded.target() {
            Target::Empty => {
                if root.id() != 0 || root.descriptor().is_some() || root.commit_evidence().is_some()
                {
                    return Err(corrupt(
                        "World Empty source does not name an actual Empty image",
                    ));
                }
            }
            Target::Canonical(target) => {
                let entry = self.entry(target.receipt().commit_id, runtime, plan, request)?;
                if &entry.commit != target.receipt()
                    || entry.position != target.position()
                    || &entry.payload_digest != target.artifact_digest()
                    || &entry.root != target.roots()
                    || root.descriptor() != Some(&entry.root)
                    || root.commit_evidence().map(|e| e.receipt()) != Some(&entry.commit)
                {
                    return Err(corrupt(
                        "World source does not match its actual Native receipt and image",
                    ));
                }
                match root
                    .commit_evidence()
                    .ok_or_else(|| corrupt("World source has no actual Native evidence"))?
                {
                    RelationalRootCommitEvidence::Original(original) => {
                        let index = self.tails.get(&entry.commit.commit_id).ok_or_else(|| {
                            corrupt("World source original is absent from the admitted Native tail")
                        })?;
                        if !crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::matches_positioned_payload_for_request(
                            plan.tail_log[*index].positioned(), original.envelope(), request,
                        )? { return Err(corrupt("World source root is not its actual Native original")); }
                    }
                    RelationalRootCommitEvidence::Checkpoint(commit) => {
                        if commit.index() != entry.as_ref() {
                            return Err(corrupt(
                                "World source root is not its admitted Native checkpoint row",
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
fn description(cause: crate::branch::RelationalRecordedComponentRefusal) -> Stop {
    match cause {
        crate::branch::RelationalRecordedComponentRefusal::Request(cause) => Stop::Request(cause),
        crate::branch::RelationalRecordedComponentRefusal::Source(cause) => cause,
        cause => crate::durability::data::DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::CorruptCheckpoint,
            format!("World component description refused: {cause:?}"),
        )
        .into(),
    }
}
