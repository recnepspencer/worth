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
pub(crate) struct OriginalEmptySchemaObservation {
    pub(crate) generation: FoundationalBranchReferenceGeneration,
    pub(crate) truth_version: RelationalBranchVersion,
    pub(crate) lifecycle: RelationalBranchLifecyclePosture,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
}

#[derive(Clone, Debug)]
pub(crate) struct OriginalEmptySchemaInstallation {
    pub(crate) before: OriginalEmptySchemaObservation,
    pub(crate) after: OriginalEmptySchemaObservation,
    /// Initial installation is possible only before the first canonical write.
    pub(crate) follows_transition: RelationalBranchTransitionSequence,
}

/// The actual performed transition supplies its own branch name and C/T axes.
#[derive(Clone, Debug)]
pub(crate) struct RetainedBranchTransition {
    pub(crate) positioned: Arc<PositionedBranchTransition>,
    pub(crate) schema: Arc<RelationalBranchRootSchemaAuthority>,
}

impl OriginalEmptySchemaObservation {
    pub(crate) fn from_empty_cell(cell: &RelationalBranchReferenceCell) -> Option<Self> {
        let (generation, truth_version, lifecycle, schema) = cell.retained_empty_schema_data()?;
        Some(Self {
            generation,
            truth_version,
            lifecycle,
            schema,
        })
    }
}
