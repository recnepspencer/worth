//! Reconstruct only the selector DATA actually recorded by canonical publication.
use super::{RequiredOutputPublicationPositions, RequiredOutputRoutePosition};
use crate::history::data::CanonicalCommitEnvelope;
use crate::transactions::data::TransactionCommitError;
use worth_execution::ExecutionRequest;

impl RequiredOutputPublicationPositions {
    pub(crate) fn from_original_commit(
        previous: &Self,
        envelope: &CanonicalCommitEnvelope,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        if !envelope.is_unmerged_output_publication_step() {
            return Ok(Self::unresolved_at(envelope.commit.commit_id));
        }
        let result = execution
            .in_scope(|lease| {
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                    .run_with_checkpoint(lease, None, |context| {
                        let mut positions = previous.clone();
                        if let Some(origin) = &envelope.required_output_origin {
                            positions.insert(
                                &origin.meaning,
                                RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
                                lease,
                                context,
                            )?;
                        }
                        for retirement in &envelope.required_output_codec_retirements {
                            positions.insert(
                                &retirement.meaning,
                                RequiredOutputRoutePosition::CodecRetired(
                                    envelope.commit.commit_id,
                                ),
                                lease,
                                context,
                            )?;
                        }
                        Ok::<_, TransactionCommitError>(positions)
                    })
            })
            .map_err(super::request)?
            .map_err(super::request)?
            .0;
        execution.consult().map_err(super::request)?;
        result
    }
}
