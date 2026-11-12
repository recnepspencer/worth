//! Completed executions may have sound facts that cannot be indexed.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;

/// Both variants prove this exact accepted row at its selected basis. Full
/// comparison never manufactures a Clean actor row or its cleanup authority.
pub(super) enum CurrentAcceptedBacking<'basis> {
    Marked {
        _cleanup: VerifiedCurrentCleanup,
    },
    FullyVerified {
        _proof: FullyVerifiedAcceptedFacts<'basis>,
    },
}
pub(super) struct FullyVerifiedAcceptedFacts<'basis> {
    _selected: &'basis PositionedRelationalSnapshot,
}
impl From<VerifiedCurrentCleanup> for CurrentAcceptedBacking<'_> {
    fn from(cleanup: VerifiedCurrentCleanup) -> Self {
        Self::Marked { _cleanup: cleanup }
    }
}

impl AcceptedCurrentCandidate {
    pub(super) fn certify_executed_full_current<'basis>(
        &'basis self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        product: &'basis WorthQueryProductObservationLease,
        snapshot: &'basis SnapshotHandle,
        selected: &'basis PositionedRelationalSnapshot,
        facts: &super::super::super::ComparableSourceFacts,
    ) -> Result<Option<CurrentAcceptedOutput<'basis>>, CurrentAcceptedStop> {
        let recorded = self.selected.recorded();
        if !recorded.computation_source.has_completed_execution()
            || !matches!(
                owner
                    .currentness(selected, &recorded.settlement_identity)
                    .map_err(|stop| CurrentAcceptedStop::Registration(stop.into()))?,
                SourceSettlementCurrentness::FullVerificationRequired(
                    FullVerificationReason::UnsupportedFact
                )
            )
        {
            return Ok(None);
        }
        let Some(witness) = self.selected.native_output_witness_cell() else {
            return Ok(None);
        };
        // This established owner verifies all facts and the complete consumed
        // graph, including its Native witness and each read's authorization.
        // Unsupported postings remain unsupported after this read.
        let verified = ConsumedOutputEvidence::verify_at_observation(
            execution,
            &recorded.settlement_identity,
            facts,
            &recorded.consumed_outputs,
            Some(FullVerificationReason::UnsupportedFact),
            witness,
            owner,
            runtime,
            snapshot,
            selected,
        );
        let verified = match verified {
            Ok(verified) => verified,
            Err(ConsumedOutputVerificationStop::PendingUpstream) => return Ok(None),
            Err(cause) => return Err(CurrentAcceptedStop::Closure(cause)),
        };
        if verified != ConsumedOutputVerification::Current {
            return Ok(None);
        }
        Ok(Some(CurrentAcceptedOutput {
            _computation: facts.computation(),
            candidate: self,
            product,
            _snapshot: snapshot,
            _selected: selected,
            facts: facts.clone(),
            _currentness: CurrentAcceptedBacking::FullyVerified {
                _proof: FullyVerifiedAcceptedFacts {
                    _selected: selected,
                },
            },
        }))
    }
}
