//! Immutable schema custody retained by the original branch lifecycle.
use super::{PositionedBranchTransition, RelationalBranchTransitionSequence};
use crate::branch::lifecycle::RelationalBranchLifecyclePosture;
use crate::branch::reference::RelationalBranchReferenceCell;
use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::branch::version::RelationalBranchVersion;
use std::sync::Arc;
use worth_foundational::FoundationalBranchReferenceGeneration;

/// The branch name comes from the same source's existing main-branch binding.
/// Empty live branches cannot be forked, and main cannot be deleted.
#[derive(Clone, Debug)]
pub struct OriginalEmptySchemaObservation {
    pub generation: FoundationalBranchReferenceGeneration,
    pub truth_version: RelationalBranchVersion,
    pub lifecycle: RelationalBranchLifecyclePosture,
    pub schema: Arc<RelationalBranchRootSchemaAuthority>,
}

#[derive(Clone, Debug)]
pub struct OriginalEmptySchemaInstallation {
    pub before: OriginalEmptySchemaObservation,
    pub after: OriginalEmptySchemaObservation,
    /// Initial installation is possible only before the first canonical write.
    pub follows_transition: RelationalBranchTransitionSequence,
}

/// The actual performed transition supplies its own branch name and C/T axes.
#[derive(Clone, Debug)]
pub struct RetainedBranchTransition {
    pub positioned: Arc<PositionedBranchTransition>,
    pub schema: Arc<RelationalBranchRootSchemaAuthority>,
}

impl OriginalEmptySchemaObservation {
    pub fn from_empty_cell(cell: &RelationalBranchReferenceCell) -> Option<Self> {
        let (generation, truth_version, lifecycle, schema) = cell.retained_empty_schema_data()?;
        Some(Self {
            generation,
            truth_version,
            lifecycle,
            schema,
        })
    }
}
