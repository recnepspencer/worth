//! Explicit model correspondence across two owner runtime frames.
use super::*;
use crate::branch::{
    relational_branch_observation, RelationalBranchCellCheckpoint, RelationalBranchCellDenial,
    RelationalBranchIdentity, RelationalBranchReferenceObservation,
};
use crate::history::data::BranchId;

impl RelationalCheckpointInspection<'_> {
    /// Compare reconstructed model meaning across the supplied owner frames.
    /// All ordinary model fields keep their original exact encodings. Only
    /// branch-cell runtime identity is rebound, after validating the complete
    /// original observations and fork provenance. This descriptive comparison
    /// grants no basis, custody or recovery authority.
    /// Cost: O(B + C + P) comparison for B non-cell bytes, C branch cells and
    /// P parent references; canonical cell copies retain O(P) temporary data.
    pub fn model_matches_rebound_runtime(
        &self,
        other: &RelationalCheckpointInspection<'_>,
        source: &RelationalBranchIdentity,
        recovered: &RelationalBranchIdentity,
    ) -> Result<bool, RelationalBranchCellDenial> {
        for cell in self.branch_cells.iter().chain(&other.branch_cells) {
            validate_cell(cell)?;
        }
        if !in_frame(&self.branch_cells, source)
            || !in_frame(&other.branch_cells, recovered)
            || self.model.envelope != other.model.envelope
            || self.model.allocator != other.model.allocator
            || !self
                .model
                .fields
                .iter()
                .filter(|(key, _)| key.as_str() != "branch_cells")
                .eq(other
                    .model
                    .fields
                    .iter()
                    .filter(|(key, _)| key.as_str() != "branch_cells"))
            || self.branch_cells.len() != other.branch_cells.len()
        {
            return Ok(false);
        }
        for (left, right) in self.branch_cells.iter().zip(&other.branch_cells) {
            let left = left
                .in_canonical_frame()
                .map_err(|_| RelationalBranchCellDenial::CheckpointObservationMismatch)?;
            let right = right
                .in_canonical_frame()
                .map_err(|_| RelationalBranchCellDenial::CheckpointObservationMismatch)?;
            if left != right {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn in_frame(cells: &[RelationalBranchCellCheckpoint], frame: &RelationalBranchIdentity) -> bool {
    cells
        .iter()
        .all(|cell| cell.runtime_instance_id == frame.runtime_instance_id())
}

fn validate_cell(cell: &RelationalBranchCellCheckpoint) -> Result<(), RelationalBranchCellDenial> {
    if !observation_matches_frame(cell.runtime_instance_id, &cell.branch_id, &cell.observation) {
        return Err(RelationalBranchCellDenial::CheckpointObservationMismatch);
    }
    match (&cell.fork_provenance, &cell.fork_source_branch_id) {
        (None, None) => Ok(()),
        (Some(observation), Some(branch)) => {
            if observation_matches_frame(cell.runtime_instance_id, branch, observation) {
                Ok(())
            } else {
                Err(RelationalBranchCellDenial::CheckpointForkProvenanceMismatch)
            }
        }
        (Some(_), None) | (None, Some(_)) => {
            Err(RelationalBranchCellDenial::CheckpointForkProvenanceMismatch)
        }
    }
}

fn observation_matches_frame(
    runtime: u64,
    branch: &BranchId,
    observation: &RelationalBranchReferenceObservation,
) -> bool {
    let expected = relational_branch_observation(
        runtime,
        &branch.0,
        observation.target().clone(),
        observation.generation(),
    );
    expected.is_ok_and(|expected| &expected == observation)
}

#[cfg(test)]
mod tests;
