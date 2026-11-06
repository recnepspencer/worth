//! Canonical authoritative payload encoding retains legacy bytes for ordinary commits.
use super::CanonicalCommitEnvelope;
impl CanonicalCommitEnvelope {
    pub(super) fn write_authoritative_payload<W: std::io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), rmp_serde::encode::Error> {
        let identity_and_authority = (
            &self.commit,
            &self.branch_context,
            &self.branch_cell_checkpoint,
            &self.authority_kind,
            &self.strategy_artifacts,
            &self.merge_execution_authority,
            &self.merge_parent_branches,
            &self.merge_base_commits,
        );
        let canonical_semantics = (
            &self.required_output_origin,
            &self.authored_cascade_retirement,
            &self.authored_cascade_restoration,
            &self.required_output_materialization,
            &self.schema_version,
            &self.schema_authority,
            &self.merged_plan,
            &self.record_allocations,
            &self.patch,
            &self.descriptive_touches,
            &self.lineage,
            &self.schema_transition,
            &self.schema_continuation_descriptor,
            &self.schema_reconciliation_descriptor,
            &self.descriptor_semantics_version,
        );
        if let Some(group) = &self.coupled_authored_publication {
            return rmp_serde::encode::write_named(
                writer,
                &(
                    identity_and_authority,
                    canonical_semantics,
                    &self.authored_index_writes,
                    group,
                ),
            );
        }
        if let Some(event) = &self.producer_cascade_retirement {
            return rmp_serde::encode::write_named(
                writer,
                &(
                    identity_and_authority,
                    canonical_semantics,
                    &self.authored_index_writes,
                    event,
                ),
            );
        }
        if self.authored_index_writes.is_unavailable() {
            rmp_serde::encode::write_named(writer, &(identity_and_authority, canonical_semantics))
        } else {
            rmp_serde::encode::write_named(
                writer,
                &(
                    identity_and_authority,
                    canonical_semantics,
                    &self.authored_index_writes,
                ),
            )
        }
    }
}
