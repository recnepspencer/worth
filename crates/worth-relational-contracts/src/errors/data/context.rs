use serde::{Deserialize, Serialize};

use crate::identity::data::VersionId;
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalSubsystem {
    Transaction,
    Durability,
    History,
    Schema,
    Publication,
    Replay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorOperation {
    Commit,
    Validate,
    ApplyMutation,
    Publish,
    ResolveSchema,
    CreateBranch,
    Replay,
    Recover,
    ReadDurableStore,
    WriteDurableStore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestedFix {
    Retry,
    RefreshSnapshot,
    RebuildFromCanonicalArtifacts,
    ValidateSchemaRegistration,
    VerifyBranchInputs,
    RepairDurableStore,
    InspectDiagnostics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorContext {
    pub subsystem: RelationalSubsystem,
    pub operation: ErrorOperation,
    pub affected_records: Vec<RecordRef>,
    pub version_context: Option<VersionId>,
    pub suggested_fix: Option<SuggestedFix>,
}

impl ErrorContext {
    pub fn new(subsystem: RelationalSubsystem, operation: ErrorOperation) -> Self {
        Self {
            subsystem,
            operation,
            affected_records: Vec::new(),
            version_context: None,
            suggested_fix: None,
        }
    }

    pub fn with_records(mut self, affected_records: Vec<RecordRef>) -> Self {
        self.affected_records = affected_records;
        self
    }

    pub fn with_version(mut self, version_context: VersionId) -> Self {
        self.version_context = Some(version_context);
        self
    }

    pub fn with_fix(mut self, suggested_fix: SuggestedFix) -> Self {
        self.suggested_fix = Some(suggested_fix);
        self
    }
}
