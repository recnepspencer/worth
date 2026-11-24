//! Check the independent writers through the one current-custody reader on the complete candidate root.
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyFieldsPort as _,
};

pub(crate) fn validate(
    execution: &ExecutionRequest<'_, '_>,
    root: &RelationalBranchRoot,
    candidate: &CanonicalCommitEnvelope,
) -> Result<(), TransactionCommitError> {
    let Some(event) = candidate.owner_historical_output_selection() else {
        return Ok(());
    };
    let _funding = execution
        .reserve_retained_memory(
            ((event.publications.len() + event.absences.len()) as u64)
                .saturating_mul(4 * std::mem::size_of::<RecordRef>() as u64),
        )
        .map_err(request)?;
    let records = event
        .publications
        .iter()
        .map(|origin| RecordRef::Entity(origin.meaning.source_scope))
        .chain(
            event
                .absences
                .iter()
                .map(|absence| RecordRef::Entity(absence.meaning.source_scope)),
        )
        .collect::<Vec<_>>();
    execution
        .run_native_with_checkpoint(|context| {
            read_candidate(execution, root, &records, context).map(drop)
        })
        .map_err(request)?
        .0
}
