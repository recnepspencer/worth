//! Original source coordinates resolve only already-folded exact observations.
use super::*;
use crate::branch::{RecordedRelationalComponentTarget as Target, RelationalBranchBasisPosture};
impl<'a> WorldSources<'a> {
    pub(super) fn source(
        &self,
        fold: &Fold<'a>,
        source: &'a Source,
    ) -> Result<(&'a BranchId, Axes<'a>), Refusal> {
        match source {
            Source::Recorded(recorded) => {
                let descriptor = recorded.descriptor();
                let state = fold
                    .observations
                    .get(&ObservationKey::of_descriptor(descriptor))
                    .ok_or_else(|| {
                        malformed("World source has no exact folded Native observation")
                    })?;
                if state.removed
                    || descriptor.posture() != RelationalBranchBasisPosture::Live
                    || !state.matches_target(descriptor.reference().target())
                    || self.schemas.get(&state.schema) != Some(&descriptor.schema_commitment())
                {
                    return Err(malformed("World source descriptor disagrees with its original Native schema or target"));
                }
                match recorded.target() {
                    Target::Empty => {
                        if state.commit.is_some() || descriptor.visibility_commitment() != [0; 32] {
                            return Err(Refusal::Malformed);
                        }
                    }
                    Target::Canonical(target) => {
                        let index = state.commit.ok_or(Refusal::Malformed)?;
                        if target.receipt() != &index.commit
                            || target.position() != index.position
                            || target.artifact_digest() != &index.payload_digest
                            || target.roots() != &index.root
                        {
                            return Err(Refusal::Malformed);
                        }
                        // This is DATA eligibility only. Actual selected root,
                        // visibility and original body admission still precede
                        // Native/World operational capability issuance.
                    }
                }
                Ok((descriptor.branch_id(), state.clone()))
            }
            Source::Published(coordinates) => {
                let state = fold
                    .canonical
                    .get(&coordinates.commit())
                    .ok_or(Refusal::Malformed)?;
                let index = state.commit.ok_or(Refusal::Malformed)?;
                if index.position != coordinates.position()
                    || &index.payload_digest != coordinates.digest().bytes()
                {
                    return Err(Refusal::Malformed);
                }
                Ok((&index.commit.branch_id, state.clone()))
            }
        }
    }
}
