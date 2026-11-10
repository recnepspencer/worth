//! Branch-authored history discovery admitted against one unchanged publication cut.
use crate::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial, RelationalBranchIdentity,
};
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{
    HistoricalLineageResolution, HistoricalResolutionBoundednessBasis, HistoricalResolutionRequest,
};
use crate::runtime::{CanonicalCheckpointAdmissionError, RelationalRuntime};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub enum RelationalLineageDiscoveryDenial {
    Branch(RelationalBranchBasisDenial),
    Execution(RelationalExecutionDenialCause),
    PublicationInFlight,
    PerformedPublicationRequiresSettlement(CommitId),
}
impl From<CanonicalCheckpointAdmissionError> for RelationalLineageDiscoveryDenial {
    fn from(cause: CanonicalCheckpointAdmissionError) -> Self {
        match cause {
            CanonicalCheckpointAdmissionError::PublicationInFlight => Self::PublicationInFlight,
            CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(commit) => {
                Self::PerformedPublicationRequiresSettlement(commit)
            }
        }
    }
}

impl RelationalRuntime {
    /// Discover the replacement/split/merge chain authored by this branch.
    /// The admitted branch binds ownership, not selected-commit ancestry.
    /// Publication drift refuses this attempt; a later call may discover more events.
    pub fn discover_authored_lineage_history(
        &self,
        request: &ExecutionRequest<'_, '_>,
        branch: &RelationalBranchIdentity,
        seed: LineageId,
    ) -> Result<HistoricalLineageResolution, RelationalLineageDiscoveryDenial> {
        self.discover_at_validated_cut(request, branch, |_| {
            self.lineage_access().resolve_historical_lineage(
                request,
                HistoricalResolutionRequest {
                    branch_id: branch.branch_id().clone(),
                    lineage_id: seed,
                    boundedness_basis:
                        HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                },
            )
        })
    }

    /// Discover branch-authored history from a record's exact Head association.
    pub fn discover_authored_record_history(
        &self,
        request: &ExecutionRequest<'_, '_>,
        branch: &RelationalBranchIdentity,
        entity: EntityId,
    ) -> Result<Option<HistoricalLineageResolution>, RelationalLineageDiscoveryDenial> {
        self.discover_at_validated_cut(request, branch, |basis| {
            let node =
                self.lineage_node_for_admitted_observation(request, &basis.observation(), entity)?;
            let Some(node) = node else {
                return Ok(None);
            };
            self.lineage_access()
                .resolve_historical_lineage(
                    request,
                    HistoricalResolutionRequest {
                        branch_id: branch.branch_id().clone(),
                        lineage_id: node.node().lineage_id(),
                        boundedness_basis:
                            HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
                    },
                )
                .map(Some)
        })
    }

    fn discover_at_validated_cut<T>(
        &self,
        request: &ExecutionRequest<'_, '_>,
        branch: &RelationalBranchIdentity,
        discover: impl FnOnce(
            &AdmittedRelationalBranchBasis,
        ) -> Result<T, RelationalExecutionDenialCause>,
    ) -> Result<T, RelationalLineageDiscoveryDenial> {
        crate::execution::check_request(request)
            .map_err(RelationalLineageDiscoveryDenial::Execution)?;
        let routes = self.history.canonical_checkpoint_gate();
        let selection = self
            .lineage
            .with_discovery_metadata(|| routes.checkpoint_selection())
            .map_err(RelationalLineageDiscoveryDenial::from)?;
        let (_, basis) = self
            .observe_branch(branch)
            .map_err(RelationalLineageDiscoveryDenial::Branch)?;
        // Provider I/O and traversal run outside the publication gate.
        let result = discover(&basis);
        self.lineage
            .with_discovery_metadata(|| routes.validate_checkpoint_selection(&selection))
            .map_err(RelationalLineageDiscoveryDenial::from)?;
        result.map_err(RelationalLineageDiscoveryDenial::Execution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lineage_discovery_preserves_pending_and_concurrent_selection_denials() {
        assert!(matches!(
            RelationalLineageDiscoveryDenial::from(
                CanonicalCheckpointAdmissionError::PublicationInFlight
            ),
            RelationalLineageDiscoveryDenial::PublicationInFlight
        ));
        assert!(matches!(
            RelationalLineageDiscoveryDenial::from(
                CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(
                    CommitId(7)
                )
            ),
            RelationalLineageDiscoveryDenial::PerformedPublicationRequiresSettlement(CommitId(7))
        ));
    }
}
