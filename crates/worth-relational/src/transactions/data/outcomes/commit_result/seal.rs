//! Consume the one original live seal, including its shared semantic value.
use super::CommitResult;

impl CommitResult {
    pub(crate) fn from_authoritative_commit(
        seal: crate::authority::commit::CommitResultSeal,
    ) -> Self {
        let (
            semantic_result,
            outcome,
            summary,
            structural_summary,
            schema_summary,
            publication,
            validation,
            execution,
            created_entities,
            created_relations,
            late_interruption,
        ) = seal.into_parts();
        Self {
            semantic_result,
            outcome,
            summary,
            structural_summary,
            schema_summary,
            publication,
            validation,
            execution,
            created_entities,
            created_relations,
            late_interruption,
        }
    }
}
