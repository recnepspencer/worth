//! Join World descriptions to original Native artifacts or admitted image rows.
use super::*;
use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
use contracts::history::data::coordinates::CommitId;
use worth_relational_branch::internal_port::branch::recorded_basis::{
    RecordedRelationalComponentReferenceOwnerPort as _, RecordedRelationalComponentTarget as Target,
};
use worth_relational_branch::internal_port::branch::root::commit_evidence::RelationalRootCommitEvidence;
use worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex;
use worth_relational_contracts::internal_port as contracts;

impl WorldSelectionReplay {
    pub(super) fn entry(
        &self,
        commit: CommitId,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetainedCanonicalIndex, Stop> {
        let artifact = inputs
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
            RetainedCanonicalIndex::from_publication(
                &positioned,
                artifact.roots(),
                digest,
                request,
                readmit_journal_commit,
            )
            .map_err(|cause| {
                contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::from(cause).into()
            })
        } else {
            let (entry, source) = artifact
                .envelope_residency()
                .source_placement()
                .ok_or_else(|| {
                    corrupt("World component has no selected Native checkpoint source")
                })?;
            let admitted = inputs
                .checkpoint
                .admitted_commit_for_request(&entry, request)?
                .ok_or_else(|| {
                    corrupt("World component lacks an admitted Native checkpoint row")
                })?;
            if !inputs
                .checkpoint
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
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        roots: &RecoveredRootInventory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Recorded, Stop> {
        self.recorded_source_inner(source, inputs, plan, roots, request, false)?
            .ok_or_else(|| corrupt("World occurrence has no matching actual Native source"))
    }
    pub(super) fn recorded_source_optional(
        &self,
        source: &Source,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        roots: &RecoveredRootInventory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<Recorded>, Stop> {
        self.recorded_source_inner(source, inputs, plan, roots, request, true)
    }
    fn recorded_source_inner(
        &self,
        source: &Source,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        roots: &RecoveredRootInventory,
        request: &ExecutionRequest<'_, '_>,
        allow_absent: bool,
    ) -> Result<Option<Recorded>, Stop> {
        match source {
            Source::Recorded(recorded) => Ok(Some(recorded.clone())),
            Source::Published(coordinates) => {
                if allow_absent
                    && inputs
                        .history
                        .commit_artifact(coordinates.commit())
                        .is_none()
                {
                    return Ok(None);
                }
                let entry = self.entry(coordinates.commit(), inputs, plan, request)?;
                if coordinates.position() != entry.position
                    || coordinates.digest().bytes() != &entry.payload_digest
                {
                    return Err(corrupt(
                        "World occurrence Native source coordinates mismatch",
                    ));
                }
                let Some(observations) = roots.publication_observations(coordinates.commit())
                else {
                    if allow_absent {
                        return Ok(None);
                    }
                    return Err(corrupt("World occurrence Native source has no observation"));
                };
                let mut selected = None;
                for descriptor in observations {
                    if descriptor.branch_id() != &entry.commit.branch_id
                        || descriptor.posture()
                            != worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisPosture::Live
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
                if selected.is_none() && !allow_absent {
                    return Err(corrupt(
                        "World occurrence has no matching actual Native source",
                    ));
                }
                Ok(selected)
            }
        }
    }
    pub(super) fn require_present_recorded_entry(
        &self,
        recorded: &Recorded,
        inputs: &WorldReplayInputs<'_>,
        plan: &RecoveryPlan,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Stop> {
        let Target::Canonical(target) = recorded.target() else {
            return Ok(());
        };
        if inputs
            .history
            .commit_artifact(target.receipt().commit_id)
            .is_none()
        {
            return Ok(());
        }
        let entry = self.entry(target.receipt().commit_id, inputs, plan, request)?;
        if &entry.commit != target.receipt()
            || entry.position != target.position()
            || &entry.payload_digest != target.artifact_digest()
            || &entry.root != target.roots()
        {
            return Err(corrupt(
                "World source does not match its actual Native receipt and image",
            ));
        }
        Ok(())
    }
    pub(super) fn require_source(
        &self,
        recorded: &Recorded,
        root: &Arc<RelationalBranchRoot>,
        inputs: &WorldReplayInputs<'_>,
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
                let entry = self.entry(target.receipt().commit_id, inputs, plan, request)?;
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
fn description(
    cause: contracts::branch::recorded_basis::refusal::RelationalRecordedComponentRefusal,
) -> Stop {
    match cause {
        contracts::branch::recorded_basis::refusal::RelationalRecordedComponentRefusal::Request(
            cause,
        ) => Stop::Request(cause),
        contracts::branch::recorded_basis::refusal::RelationalRecordedComponentRefusal::Source(
            cause,
        ) => cause,
        cause => contracts::durability::data::recovery_errors::DurabilityError::new(
            contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCheckpoint,
            format!("World component description refused: {cause:?}"),
        )
        .into(),
    }
}
