use crate::diagnostics::data::{
    DiagnosticCode, RelationalDiagnosticFields, RelationalDiagnosticValue,
    RelationalDiagnosticsEntry,
};
use crate::durability::data::DurableCheckpoint;
use crate::history::data::CommitId;

pub(in crate::durability::authority) fn checkpoint_created(
    checkpoint: &DurableCheckpoint,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::CheckpointCreated,
        "durable checkpoint created",
        checkpoint_created_fields(
            checkpoint
                .coverage
                .up_to_commit
                .as_ref()
                .map(|commit| commit.commit_id),
            checkpoint.partition_images.len(),
        ),
    )
}

fn checkpoint_created_fields(
    up_to_commit: Option<CommitId>,
    partition_count: usize,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "up_to_commit",
            RelationalDiagnosticValue::optional(
                up_to_commit.map(RelationalDiagnosticValue::CommitId),
            ),
        ),
        (
            "partition_count",
            RelationalDiagnosticValue::unsigned(partition_count),
        ),
    ])
    .into()
}
