use worth_store_io_scheduler::QueueExecutionOutcome;

use crate::physical_runtime::PhysicalWorkSettlementEvidence;

use super::{
    CanonicalRecordMutationFailure, CanonicalRecordMutationSettlement,
    PreparedCanonicalRecordMutation,
};

pub(in crate::physical_runtime) struct CanonicalRecordMutationCompletion {
    physical: super::super::residency::candidate_frame_residency::CandidateFramePhysicalWrite,
}

impl CanonicalRecordMutationCompletion {
    pub(in crate::physical_runtime::record_serving) fn into_physical(
        self,
    ) -> super::super::residency::candidate_frame_residency::CandidateFramePhysicalWrite {
        self.physical
    }
}

impl PreparedCanonicalRecordMutation {
    pub(in crate::physical_runtime) fn execute(
        self,
    ) -> Result<CanonicalRecordMutationCompletion, CanonicalRecordMutationFailure> {
        let identity = self.identity;
        let retry = match &self.command {
            crate::physical_runtime::PhysicalExecutorCommand::NewArtifact(command) => {
                command.retirement_retry.clone()
            }
            _ => None,
        };
        let outcome = self
            .execution
            .execute_physical_work(self.command)
            .map_err(|failure| CanonicalRecordMutationFailure::pre_effect(identity, failure))?;
        let settled = outcome.into_settled();
        let settlement = CanonicalRecordMutationSettlement::from_settled(&settled);
        classify(self.target, settlement, settled.into_evidence(), retry)
    }
}

fn classify(
    target: crate::physical_runtime::PhysicalWorkRecoveryTarget,
    settlement: CanonicalRecordMutationSettlement,
    evidence: PhysicalWorkSettlementEvidence,
    retry: Option<super::super::RetirementCandidateRetryScope>,
) -> Result<CanonicalRecordMutationCompletion, CanonicalRecordMutationFailure> {
    match evidence {
        PhysicalWorkSettlementEvidence::Publication { physical, scheduler: QueueExecutionOutcome::Executed(_) }
            if retry.is_some() => Ok(CanonicalRecordMutationCompletion {
                physical: super::super::residency::candidate_frame_residency::CandidateFramePhysicalWrite::completed_retirement_retry(physical, settlement, retry.unwrap()),
            }),
        PhysicalWorkSettlementEvidence::NewArtifact {
            physical,
            coordinate,
            scheduler: QueueExecutionOutcome::Executed(_),
        } => Ok(CanonicalRecordMutationCompletion {
            physical: super::super::residency::candidate_frame_residency::
                CandidateFramePhysicalWrite::completed(physical, coordinate, settlement),
        }),
        PhysicalWorkSettlementEvidence::Publication {
            physical,
            scheduler: QueueExecutionOutcome::Executed(_),
        } if matches!(physical.coordinate().artifact(), worth_store_physical_format::RecordArtifactFile::ExtentArena { .. }) => Ok(CanonicalRecordMutationCompletion {
            physical: super::super::residency::candidate_frame_residency::CandidateFramePhysicalWrite::completed_arena_range(physical, settlement),
        }),
        PhysicalWorkSettlementEvidence::NoEffect(evidence) => Err(
            CanonicalRecordMutationFailure::backend(settlement, target, evidence.failure()),
        ),
        PhysicalWorkSettlementEvidence::TerminalFailure(failure) => {
            Err(CanonicalRecordMutationFailure::terminal(settlement, failure))
        }
        _ => Err(CanonicalRecordMutationFailure::settlement_mismatch(
            settlement,
        )),
    }
}
