//! The canonical tuple projection used for authoritative byte observations.
use super::CanonicalCommitEnvelopeBody;

impl CanonicalCommitEnvelopeBody {
    /// Canonical bytes for owner truth only. Diagnostics and optional derived
    /// indexes have distinct lifecycle lanes and cannot perturb commit-byte
    /// accounting.
    #[cfg(test)]
    pub(crate) fn encode_authoritative_payload(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let mut bytes = Vec::new();
        self.write_authoritative_payload(&mut bytes)?;
        Ok(bytes)
    }

    pub(crate) fn write_authoritative_payload<W: std::io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), rmp_serde::encode::Error> {
        let identity_and_authority = (
            &self.index_definition_delta,
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
