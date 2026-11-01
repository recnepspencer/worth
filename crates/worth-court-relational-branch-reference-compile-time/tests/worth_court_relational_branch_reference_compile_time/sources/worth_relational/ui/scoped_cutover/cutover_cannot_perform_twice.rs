use worth_execution::ExecutionRequest;
use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, RelationalPublicationPort,
};

fn deny(
    port: &RelationalPublicationPort,
    request: &ExecutionRequest<'_, '_>,
    candidate: PreparedRelationalCommitCandidate,
) {
    let _ = port.with_prepared_memory_cutover(request, candidate, |cutover| {
        cutover.perform();
        cutover.perform();
    });
}

fn main() {}
