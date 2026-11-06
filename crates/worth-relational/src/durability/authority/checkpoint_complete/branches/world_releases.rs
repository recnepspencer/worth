//! Preserve canonical originals while restoring only Home-admitted released sources.
use super::*;
use crate::durability::{
    RelationalWorldOriginalDescription, RelationalWorldSourceDescription as Source,
};
use worth_foundational::WorldOriginalFate;

pub(super) struct WorldReleases<'a> {
    by_commit: ExecutionOrderedMap<CommitId, &'a RelationalWorldOriginalDescription>,
    restorations: ExecutionOrderedMap<CommitId, ReleasedBranchAxes<'a>>,
}

impl<'a> WorldReleases<'a> {
    pub(super) fn prepare(
        original: &'a OriginalBranchSchemaInputs,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut result = Self {
            by_commit: Default::default(),
            restorations: Default::default(),
        };
        if let Some(selection) = &original.home_world_selection {
            for row in selection.source().originals() {
                match row.fate() {
                    WorldOriginalFate::Open
                    | WorldOriginalFate::Continued
                    | WorldOriginalFate::Published => continue,
                    WorldOriginalFate::Released => {}
                }
                let Some(effect) = row.effect() else { continue };
                if result.by_commit.contains_key(&effect.commit()) {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                result
                    .by_commit
                    .insert_cloned_native_for_request(&effect.commit(), &row, *request)
                    .map_err(map_stop)?;
            }
        }
        Ok(result)
    }

    pub(super) fn retain_original(
        &mut self,
        commit: &'a OriginalCanonicalCommit,
        before: Axes,
        after: Axes,
        commits: &[OriginalCanonicalCommit],
        roots: &FoldedRoots,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if self
            .admits_original(commit, &before, commits, roots)?
            .is_none()
        {
            return Ok(());
        }
        let id = commit.envelope.commit.commit_id;
        let restoration = ReleasedBranchAxes {
            branch: &commit.envelope.branch_context,
            before,
            after,
        };
        self.restorations
            .insert_cloned_native_for_request(&id, &restoration, *request)
            .map_err(map_stop)
    }
    pub(super) fn predecessor(&self, branch: &BranchId, current: &Axes) -> Option<&Axes> {
        let saved = self.restorations.get(&current.commit?)?;
        (saved.branch == branch && current.same_state(&saved.after)).then_some(&saved.before)
    }
    pub(super) fn restorations(&self) -> impl Iterator<Item = &ReleasedBranchAxes<'a>> {
        self.restorations.values()
    }
    fn admits_original(
        &self,
        commit: &OriginalCanonicalCommit,
        before: &Axes,
        commits: &[OriginalCanonicalCommit],
        roots: &FoldedRoots,
    ) -> Result<Option<&'a RelationalWorldOriginalDescription>, Denial> {
        use sha2::{Digest, Sha256};
        let Some(row) = self.by_commit.get(&commit.envelope.commit.commit_id) else {
            return Ok(None);
        };
        let effect = row.effect().ok_or(Denial::OriginalAuthorityUnavailable)?;
        let caller = commit
            .envelope
            .caller_association()
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        if worth_foundational::HomeProgramTransitionCaller::recognizes(caller.as_bytes())
            || row.caller().bytes() != &<[u8; 32]>::from(Sha256::digest(caller.as_bytes()))
            || row.receipt() != Some(&commit.envelope.commit)
            || effect.position() != commit.position
            || effect.digest().bytes() != &commit.original_payload_digest
            || !source_matches(
                row.source(),
                &commit.envelope.branch_context,
                before,
                commits,
                roots,
            )
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        Ok(Some(*row))
    }
}

fn source_matches(
    source: &Source,
    branch: &BranchId,
    before: &Axes,
    commits: &[OriginalCanonicalCommit],
    roots: &FoldedRoots,
) -> bool {
    match source {
        Source::Recorded(recorded) => {
            let descriptor = recorded.descriptor();
            let visibility = before
                .commit
                .and_then(|id| roots.get(id))
                .and_then(|row| row.root.axes())
                .map_or([0; 32], |axes| axes.visibility.digest());
            descriptor.branch_id() == branch
                && descriptor.posture() == RelationalBranchBasisPosture::Live
                && descriptor.schema_commitment() == before.schema.authority_digest()
                && descriptor.visibility_commitment() == visibility
                && before.matches(
                    descriptor.reference().generation(),
                    descriptor.truth_version(),
                    descriptor.reference().target(),
                    roots,
                )
                && match recorded.target() {
                    crate::branch::RecordedRelationalComponentTarget::Empty => {
                        before.commit.is_none()
                    }
                    crate::branch::RecordedRelationalComponentTarget::Canonical(target) => {
                        canonical(before, target.position(), commits).is_some_and(|actual| {
                            &actual.envelope.commit == target.receipt()
                                && actual.artifact.roots() == target.roots()
                                && &actual.original_payload_digest == target.artifact_digest()
                        })
                    }
                }
        }
        Source::Published(coordinates) => canonical(before, coordinates.position(), commits)
            .is_some_and(|actual| {
                actual.envelope.commit.commit_id == coordinates.commit()
                    && &actual.original_payload_digest == coordinates.digest().bytes()
            }),
    }
}

fn canonical<'a>(
    before: &Axes,
    position: crate::publication::patch::data::PatchStreamPosition,
    commits: &'a [OriginalCanonicalCommit],
) -> Option<&'a OriginalCanonicalCommit> {
    let index = usize::try_from(position.0.checked_sub(1)?).ok()?;
    commits.get(index).filter(|actual| {
        actual.position == position && Some(actual.envelope.commit.commit_id) == before.commit
    })
}

#[derive(Clone)]
pub(super) struct ReleasedBranchAxes<'a> {
    pub(super) branch: &'a BranchId,
    pub(super) before: Axes,
    pub(super) after: Axes,
}
