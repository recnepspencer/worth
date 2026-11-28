//! Faults in the previously authenticated publication carried by a real writer.
use super::super::super::custody::{self, RelationalRequiredOutputPublicationFixturePort as _};
use super::{reject, RelationalCoupledProducerPosture, RequiredOutputPublication};
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::CodecRetirementClosureStop;
use worth_relational_contracts::internal_port::{
    publication::data::authored_retirement_refusal::AuthoredRetirementRefusal,
    schema::data::RelationalSchemaRegistry,
    transactions::data::{outcomes::transaction_commit_error::TransactionCommitError, primitives::WorkerIntentBatch},
};

/// Alter only the existing predecessor custody or commands passed to Native admission.
#[doc(hidden)]
pub fn corrupt_predecessor_for_test<'a>(
    execution: &ExecutionRequest<'_, '_>,
    history: &RelationalPreparationHistory,
    registry: impl FnOnce() -> &'a RelationalSchemaRegistry,
    posture: RelationalCoupledProducerPosture,
    batches: &mut [WorkerIntentBatch],
    fault: u8,
) -> Result<RelationalCoupledProducerPosture, TransactionCommitError> {
    let RelationalCoupledProducerPosture::Publication(mut publication) = posture else {
        return Ok(posture);
    };
    if fault == 3 {
        let RequiredOutputPublication::Preserve { predecessor, .. } = &publication else {
            return Ok(RelationalCoupledProducerPosture::Publication(publication));
        };
        super::delete_membership(predecessor, registry(), batches)?;
    } else if fault == 2 {
        let RequiredOutputPublication::Preserve {
            predecessor,
            meaning,
            roles,
            ..
        } = publication
        else {
            return Ok(RelationalCoupledProducerPosture::Publication(publication));
        };
        let foreign = worth_proof::InstalledApplicationPublicationOwner::fresh();
        let admission = foreign.admit_required_output(
            predecessor.origin.producer.clone(),
            *predecessor.origin.dependency_identity.bytes(),
        );
        publication = RequiredOutputPublication::Birth {
            admission,
            absence: None,
            meaning,
            roles,
        };
    } else if let RequiredOutputPublication::Preserve { predecessor, .. } = &mut publication {
        let subject = predecessor.origin.meaning.source_scope;
        let missing = || reject(AuthoredRetirementRefusal::CorruptPublication, subject);
        let older = predecessor
            .origin
            .publication
            .predecessor()
            .ok_or_else(missing)?;
        let envelope = history
            .demand_canonical_envelope(execution, older)
            .map_err(custody::execution_stop)?
            .ok_or_else(missing)?;
        let origin = execution
            .run_native_with_checkpoint(|context| {
                envelope
                    .validate_output_origins(context)
                    .and_then(|origins| {
                        origins.output_origin_for_route(&predecessor.origin.meaning, context)
                    })
                    .map(Arc::clone)
                    .map_err(|stop| match stop {
                        CodecRetirementClosureStop::Request(cause) => {
                            custody::execution_stop(cause)
                        }
                        _ => missing(),
                    })
            })
            .map_err(custody::execution_stop)?
            .0?;
        predecessor.replace_recorded_custody_for_test(origin, envelope);
    }
    Ok(RelationalCoupledProducerPosture::Publication(publication))
}
