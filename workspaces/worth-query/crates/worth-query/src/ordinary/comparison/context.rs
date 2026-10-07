use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::ordinary::history::{at, WorthQueryHistoricalContext};
use crate::runtime::{
    WorthQueryBranchBasisAdmission, WorthQueryRuntimeBranchComparisonBasis, WorthQueryWorkspace,
};
use crate::session_label::WorthQuerySessionLabel;

use super::{
    WorthQueryComparisonJourneyCounters, WorthQueryComparisonNextAction, WorthQueryComparisonStop,
    WorthQueryComparisonStopSource,
};

/// The authority family carried by a sealed comparison basis pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryComparisonBasisFamily {
    CurrentToHistorical,
    BranchToBranch,
}

#[derive(Clone, Debug)]
pub(crate) struct WorthQueryCapturedComparisonBasis {
    workspace_name: String,
    snapshot: WorthQuerySnapshotIdentity,
}

#[derive(Clone, Debug)]
pub(crate) struct WorthQueryCapturedBranchComparisonBasis {
    workspace_name: String,
    runtime_basis: WorthQueryRuntimeBranchComparisonBasis,
}

impl WorthQueryCapturedBranchComparisonBasis {
    fn capture(
        workspace: &WorthQueryWorkspace,
        label: WorthQuerySessionLabel,
    ) -> Result<Self, crate::runtime::WorthQueryRuntimeError> {
        Ok(Self {
            workspace_name: workspace.name().to_string(),
            runtime_basis: workspace.capture_branch_comparison_basis(label)?,
        })
    }

    pub(crate) fn workspace_name(&self) -> &str {
        &self.workspace_name
    }

    pub(crate) fn snapshot(&self) -> &WorthQuerySnapshotIdentity {
        self.runtime_basis.snapshot()
    }

    pub(crate) fn admission(&self) -> &WorthQueryBranchBasisAdmission {
        self.runtime_basis.admission()
    }

    pub(crate) fn matches(
        &self,
        workspace: &WorthQueryWorkspace,
    ) -> Result<bool, worth_query_execution::facade::primary_graph::WorthQueryHandleDenial> {
        let snapshot = workspace.snapshot_identity()?;
        Ok(self.workspace_name == workspace.name()
            && self
                .runtime_basis
                .snapshot()
                .is_same_current_identity_as(&snapshot))
    }
}

impl WorthQueryCapturedComparisonBasis {
    fn capture(
        workspace: &WorthQueryWorkspace,
    ) -> Result<Self, worth_query_execution::facade::primary_graph::WorthQueryHandleDenial> {
        Ok(Self {
            workspace_name: workspace.name().to_string(),
            snapshot: workspace.snapshot_identity()?,
        })
    }

    pub(crate) fn workspace_name(&self) -> &str {
        &self.workspace_name
    }

    pub(crate) fn snapshot(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot
    }

    pub(crate) fn matches(
        &self,
        workspace: &WorthQueryWorkspace,
    ) -> Result<bool, worth_query_execution::facade::primary_graph::WorthQueryHandleDenial> {
        let snapshot = workspace.snapshot_identity()?;
        Ok(self.workspace_name == workspace.name()
            && self.snapshot.is_same_current_identity_as(&snapshot))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum WorthQueryComparisonPairAuthority {
    CurrentAndRetained {
        current: WorthQueryCapturedComparisonBasis,
        retained: WorthQueryHistoricalContext,
    },
    BranchToBranch {
        left: WorthQueryCapturedBranchComparisonBasis,
        right: WorthQueryCapturedBranchComparisonBasis,
    },
}

/// A sealed structural comparison pair. Consumers choose the pair through the
/// constructors below; they cannot substitute names, digests, or snapshots.
#[derive(Clone, Debug)]
pub struct WorthQueryComparisonContext {
    pub(crate) authority: WorthQueryComparisonPairAuthority,
}

impl WorthQueryComparisonContext {
    pub fn family(&self) -> WorthQueryComparisonBasisFamily {
        match self.authority {
            WorthQueryComparisonPairAuthority::CurrentAndRetained { .. } => {
                WorthQueryComparisonBasisFamily::CurrentToHistorical
            }
            WorthQueryComparisonPairAuthority::BranchToBranch { .. } => {
                WorthQueryComparisonBasisFamily::BranchToBranch
            }
        }
    }
}

/// Pair the runtime's current truth with its exact retained historical view.
pub fn current_and_retained(
    workspace: &WorthQueryWorkspace,
) -> Result<
    WorthQueryComparisonContext,
    worth_query_execution::facade::primary_graph::WorthQueryHandleDenial,
> {
    Ok(WorthQueryComparisonContext {
        authority: WorthQueryComparisonPairAuthority::CurrentAndRetained {
            current: WorthQueryCapturedComparisonBasis::capture(workspace)?,
            retained: at(workspace)?,
        },
    })
}

/// Structurally bind two independently owned workspace/branch bases.
///
/// The returned context is not sufficient by itself: execution rechecks both
/// typed snapshots against the exact workspace names before either query runs.
pub fn between(
    left: &WorthQueryWorkspace,
    left_label: WorthQuerySessionLabel,
    right: &WorthQueryWorkspace,
    right_label: WorthQuerySessionLabel,
) -> Result<WorthQueryComparisonContext, WorthQueryComparisonStop> {
    let counters = WorthQueryComparisonJourneyCounters::validate_pair();
    if left.name() == right.name() {
        return Err(WorthQueryComparisonStop::new(
            WorthQueryComparisonStopSource::InvalidBasisPair,
            WorthQueryComparisonNextAction::RefreshBasisPair,
            "branch comparison requires two independently named workspace authorities",
            counters,
        ));
    }
    let left =
        WorthQueryCapturedBranchComparisonBasis::capture(left, left_label).map_err(|error| {
            WorthQueryComparisonStop::new(
                match &error {
                    crate::runtime::WorthQueryRuntimeError::Workspace(error) => {
                        match error.kind() {
                            crate::memory_workspace::WorthQueryWorkspaceErrorKind::Handle(
                                denial,
                            ) => WorthQueryComparisonStopSource::Handle(denial),
                            _ => WorthQueryComparisonStopSource::LeftBasisAdmission,
                        }
                    }
                    _ => WorthQueryComparisonStopSource::LeftBasisAdmission,
                },
                WorthQueryComparisonNextAction::ResolveAuthority,
                format!("left branch basis admission failed: {error:?}"),
                counters.clone(),
            )
        })?;
    let right =
        WorthQueryCapturedBranchComparisonBasis::capture(right, right_label).map_err(|error| {
            WorthQueryComparisonStop::new(
                match &error {
                    crate::runtime::WorthQueryRuntimeError::Workspace(error) => {
                        match error.kind() {
                            crate::memory_workspace::WorthQueryWorkspaceErrorKind::Handle(
                                denial,
                            ) => WorthQueryComparisonStopSource::Handle(denial),
                            _ => WorthQueryComparisonStopSource::RightBasisAdmission,
                        }
                    }
                    _ => WorthQueryComparisonStopSource::RightBasisAdmission,
                },
                WorthQueryComparisonNextAction::ResolveAuthority,
                format!("right branch basis admission failed: {error:?}"),
                counters.clone(),
            )
        })?;
    if left.admission().admission_identity() == right.admission().admission_identity() {
        return Err(WorthQueryComparisonStop::new(
            WorthQueryComparisonStopSource::InvalidBasisPair,
            WorthQueryComparisonNextAction::RefreshBasisPair,
            "branch comparison requires two distinct admitted branch authorities",
            counters,
        ));
    }
    Ok(WorthQueryComparisonContext {
        authority: WorthQueryComparisonPairAuthority::BranchToBranch { left, right },
    })
}
