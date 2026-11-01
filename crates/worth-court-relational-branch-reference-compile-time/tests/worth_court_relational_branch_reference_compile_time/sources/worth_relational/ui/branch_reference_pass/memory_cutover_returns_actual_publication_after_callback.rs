use worth_execution::ExecutionRequest;
use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, RelationalPublicationOutcome, RelationalPublicationPort,
};

fn publish(
    port: &RelationalPublicationPort,
    request: &ExecutionRequest<'_, '_>,
    candidate: PreparedRelationalCommitCandidate,
) -> Result<Option<RelationalPublicationOutcome>, RelationalPublicationOutcome> {
    let scoped = port.with_prepared_memory_cutover(request, candidate, |cutover| {
        cutover.perform();
        Err::<(), _>("the caller's result is separate from Native's effect")
    })?;
    let (caller_result, publication) = scoped.into_parts();
    assert!(caller_result.is_err());
    Ok(publication)
}

pub(crate) fn main() {
    let _ = publish;
}
