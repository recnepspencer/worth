use crate::durability::data::DurableCheckpoint;
use contracts::diagnostics::data::artifacts::{DiagnosticCode, RelationalDiagnosticsEntry};
use contracts::diagnostics::data::fields::{RelationalDiagnosticFields, RelationalDiagnosticValue};
use contracts::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port as contracts;

pub fn checkpoint_created(checkpoint: &DurableCheckpoint) -> RelationalDiagnosticsEntry {
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
