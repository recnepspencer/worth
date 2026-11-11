//! The single v4 tuple projection of complete Original owner truth.
use super::{CanonicalCommitEnvelope, CanonicalCommitEnvelopeBody};

const ORIGINAL_ROOT_PAYLOAD_DOMAIN: &str = "worth.relational.original-root.v4";

impl CanonicalCommitEnvelope {
    #[cfg(test)]
    pub(crate) fn encode_authoritative_payload(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        let mut bytes = Vec::new();
        self.write_authoritative_payload(&mut bytes)?;
        Ok(bytes)
    }

    /// Emit the same complete tuple used by Original root ownership and visibility.
    /// Carried schema identity is ordinary; transitions additionally retain material.
    pub(crate) fn write_authoritative_payload<W: std::io::Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), rmp_serde::encode::Error> {
        let CanonicalCommitEnvelopeBody {
            content,
            index_definition_delta,
            caller_association,
            required_output_codec_retirements,
            required_output_origin,
            producer_cascade_retirement,
            authored_cascade_retirement,
            authored_cascade_restoration,
            coupled_authored_publication,
            historical_output_selection,
            required_output_materialization,
            authored_index_writes,
            commit,
            branch_context,
            branch_cell_checkpoint,
            authority_kind,
            strategy_artifacts,
            merge_execution_authority,
            merge_parent_branches,
            merge_base_commits,
            schema_version,
            schema_authority,
            merged_plan,
            symbol_texts,
            revision_symbol_references,
            record_allocations,
            patch,
            descriptive_touches,
            diagnostics_summary,
            lineage,
            derived_index_artifacts: _,
            schema_transition,
            schema_continuation_descriptor,
            schema_reconciliation_descriptor,
            descriptor_semantics_version,
        } = &self.body;
        let identity_and_authority = (
            commit,
            branch_context,
            branch_cell_checkpoint,
            authority_kind,
            strategy_artifacts,
            merge_execution_authority,
            merge_parent_branches,
            merge_base_commits,
            index_definition_delta,
            caller_association,
        );
        let publications = (
            required_output_codec_retirements,
            required_output_origin,
            producer_cascade_retirement,
            authored_cascade_retirement,
            authored_cascade_restoration,
            coupled_authored_publication,
            historical_output_selection,
            required_output_materialization,
            authored_index_writes,
        );
        let schema = (
            schema_version,
            (
                worth_foundational::facade::CanonicalDigestId::new(schema_authority.digest()),
                schema_transition.as_ref().map(|_| schema_authority),
            ),
            schema_transition,
            schema_continuation_descriptor,
            schema_reconciliation_descriptor,
            descriptor_semantics_version,
        );
        let records = (
            content,
            merged_plan,
            symbol_texts,
            revision_symbol_references,
            record_allocations,
            patch,
            descriptive_touches,
            lineage,
            (
                diagnostics_summary.scope,
                diagnostics_summary.kind,
                diagnostics_summary.determinism,
            ),
        );
        rmp_serde::encode::write_named(
            writer,
            &(
                ORIGINAL_ROOT_PAYLOAD_DOMAIN,
                identity_and_authority,
                publications,
                schema,
                records,
                self.semantic_result(),
            ),
        )
    }
}

#[cfg(test)]
#[path = "payload_encoding_tests.rs"]
mod tests;
