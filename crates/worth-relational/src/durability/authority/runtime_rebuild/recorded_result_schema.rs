//! The result schema a durable schema-changing commit recorded for itself.
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::history::data::CanonicalCommitEnvelope;
use std::sync::Arc;

/// Replay of a recorded schema-changing commit judges it under the schema it
/// durably recorded, never under its predecessor's. Only recovery issues this,
/// after the carrier matched the authority in the commit's own envelope.
#[derive(Debug)]
pub(crate) struct RecordedReplayResultSchema {
    schema: Arc<RelationalBranchRootSchemaAuthority>,
}

impl RecordedReplayResultSchema {
    pub(super) fn admit(
        recorded: Arc<RelationalBranchRootSchemaAuthority>,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<Self, DurabilityError> {
        if !recorded.matches(&envelope.schema_authority) {
            return Err(DurabilityError::new(
                RecoveryFailureClass::SchemaMismatch,
                format!(
                    "recorded result schema of commit {} differs from its recorded authority",
                    envelope.commit.commit_id.0
                ),
            ));
        }
        Ok(Self { schema: recorded })
    }

    pub(crate) fn into_schema(self) -> Arc<RelationalBranchRootSchemaAuthority> {
        self.schema
    }
}
