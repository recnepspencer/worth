//! Resolve content through the immutable read's selected root.
use super::RelationalPinnedRead;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_relational_branch::internal_port::content::{
    RelationalContentDenial, RelationalContentFrame,
};

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    pub fn read_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        format: CanonicalDigestId,
    ) -> Result<RelationalContentFrame, RelationalContentDenial> {
        self.project_read()
            .read_content(execution, identity, format)
    }
}
