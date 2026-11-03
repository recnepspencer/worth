//! Context-bearing canonical envelope decoding; construction lists every field.
use super::CanonicalEnvelopeWire;
use crate::content::decoding::{ContentDecodeContext, ContentPublicationSeed};
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::Deserializer;

pub(crate) struct CanonicalEnvelopeSeed<'a, 'r, 'c, 's>(
    pub(crate) &'a ContentDecodeContext<'r, 'c, 's>,
);
impl<'de> DeserializeSeed<'de> for CanonicalEnvelopeSeed<'_, '_, '_, '_> {
    type Value = CanonicalEnvelopeWire<'de>;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for CanonicalEnvelopeSeed<'_, '_, '_, '_> {
    type Value = CanonicalEnvelopeWire<'de>;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("canonical Native envelope map")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut content = None;
        let mut required_output_codec_retirements = None;
        let mut required_output_origin = None;
        let mut authored_cascade_restoration = None;
        let mut authored_cascade_retirement = None;
        let mut required_output_materialization = None;
        let mut authored_index_writes = None;
        let mut commit = None;
        let mut branch_context = None;
        let mut branch_cell_checkpoint = None;
        let mut authority_kind = None;
        let mut strategy_artifacts = None;
        let mut merge_execution_authority = None;
        let mut merge_parent_branches = None;
        let mut merge_base_commits = None;
        let mut schema_version = None;
        let mut schema_authority = None;
        let mut merged_plan = None;
        let mut symbol_texts = None;
        let mut revision_symbol_references = None;
        let mut record_allocations = None;
        let mut patch = None;
        let mut descriptive_touches = None;
        let mut diagnostics_summary = None;
        let mut lineage = None;
        let mut derived_index_artifacts = None;
        let mut schema_transition = None;
        let mut schema_continuation_descriptor = None;
        let mut schema_reconciliation_descriptor = None;
        let mut descriptor_semantics_version = None;
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "content" => {
                    if content.is_some() {
                        return Err(M::Error::duplicate_field("content"));
                    }
                    content = Some(map.next_value_seed(ContentPublicationSeed(self.0))?);
                }
                "required_output_codec_retirements" => {
                    if required_output_codec_retirements.is_some() {
                        return Err(M::Error::duplicate_field(
                            "required_output_codec_retirements",
                        ));
                    }
                    required_output_codec_retirements = Some(map.next_value()?);
                }
                "required_output_origin" => {
                    if required_output_origin.is_some() {
                        return Err(M::Error::duplicate_field("required_output_origin"));
                    }
                    required_output_origin = Some(map.next_value()?);
                }
                "authored_cascade_retirement" => {
                    if authored_cascade_retirement.is_some() {
                        return Err(M::Error::duplicate_field("authored_cascade_retirement"));
                    }
                    authored_cascade_retirement = Some(map.next_value()?);
                }
                "authored_cascade_restoration" => {
                    if authored_cascade_restoration.is_some() {
                        return Err(M::Error::duplicate_field("authored_cascade_restoration"));
                    }
                    authored_cascade_restoration = Some(map.next_value()?);
                }
                "required_output_materialization" => {
                    if required_output_materialization.is_some() {
                        return Err(M::Error::duplicate_field("required_output_materialization"));
                    }
                    required_output_materialization = Some(map.next_value()?);
                }
                "authored_index_writes" => {
                    if authored_index_writes.is_some() {
                        return Err(M::Error::duplicate_field("authored_index_writes"));
                    }
                    authored_index_writes = Some(map.next_value()?);
                }
                "commit" => {
                    if commit.is_some() {
                        return Err(M::Error::duplicate_field("commit"));
                    }
                    commit = Some(map.next_value()?);
                }
                "branch_context" => {
                    if branch_context.is_some() {
                        return Err(M::Error::duplicate_field("branch_context"));
                    }
                    branch_context = Some(map.next_value()?);
                }
                "branch_cell_checkpoint" => {
                    if branch_cell_checkpoint.is_some() {
                        return Err(M::Error::duplicate_field("branch_cell_checkpoint"));
                    }
                    branch_cell_checkpoint = Some(map.next_value()?);
                }
                "authority_kind" => {
                    if authority_kind.is_some() {
                        return Err(M::Error::duplicate_field("authority_kind"));
                    }
                    authority_kind = Some(map.next_value()?);
                }
                "strategy_artifacts" => {
                    if strategy_artifacts.is_some() {
                        return Err(M::Error::duplicate_field("strategy_artifacts"));
                    }
                    strategy_artifacts = Some(map.next_value()?);
                }
                "merge_execution_authority" => {
                    if merge_execution_authority.is_some() {
                        return Err(M::Error::duplicate_field("merge_execution_authority"));
                    }
                    merge_execution_authority = Some(map.next_value()?);
                }
                "merge_parent_branches" => {
                    if merge_parent_branches.is_some() {
                        return Err(M::Error::duplicate_field("merge_parent_branches"));
                    }
                    merge_parent_branches = Some(map.next_value()?);
                }
                "merge_base_commits" => {
                    if merge_base_commits.is_some() {
                        return Err(M::Error::duplicate_field("merge_base_commits"));
                    }
                    merge_base_commits = Some(map.next_value()?);
                }
                "schema_version" => {
                    if schema_version.is_some() {
                        return Err(M::Error::duplicate_field("schema_version"));
                    }
                    schema_version = Some(map.next_value()?);
                }
                "schema_authority" => {
                    if schema_authority.is_some() {
                        return Err(M::Error::duplicate_field("schema_authority"));
                    }
                    schema_authority = Some(map.next_value()?);
                }
                "merged_plan" => {
                    if merged_plan.is_some() {
                        return Err(M::Error::duplicate_field("merged_plan"));
                    }
                    merged_plan = Some(map.next_value()?);
                }
                "symbol_texts" => {
                    if symbol_texts.is_some() {
                        return Err(M::Error::duplicate_field("symbol_texts"));
                    }
                    symbol_texts = Some(map.next_value()?);
                }
                "revision_symbol_references" => {
                    if revision_symbol_references.is_some() {
                        return Err(M::Error::duplicate_field("revision_symbol_references"));
                    }
                    revision_symbol_references = Some(map.next_value()?);
                }
                "record_allocations" => {
                    if record_allocations.is_some() {
                        return Err(M::Error::duplicate_field("record_allocations"));
                    }
                    record_allocations = Some(map.next_value()?);
                }
                "patch" => {
                    if patch.is_some() {
                        return Err(M::Error::duplicate_field("patch"));
                    }
                    patch = Some(map.next_value()?);
                }
                "descriptive_touches" => {
                    if descriptive_touches.is_some() {
                        return Err(M::Error::duplicate_field("descriptive_touches"));
                    }
                    descriptive_touches = Some(map.next_value()?);
                }
                "diagnostics_summary" => {
                    if diagnostics_summary.is_some() {
                        return Err(M::Error::duplicate_field("diagnostics_summary"));
                    }
                    diagnostics_summary = Some(map.next_value()?);
                }
                "lineage" => {
                    if lineage.is_some() {
                        return Err(M::Error::duplicate_field("lineage"));
                    }
                    lineage = Some(map.next_value()?);
                }
                "derived_index_artifacts" => {
                    if derived_index_artifacts.is_some() {
                        return Err(M::Error::duplicate_field("derived_index_artifacts"));
                    }
                    derived_index_artifacts = Some(map.next_value()?);
                }
                "schema_transition" => {
                    if schema_transition.is_some() {
                        return Err(M::Error::duplicate_field("schema_transition"));
                    }
                    schema_transition = Some(map.next_value()?);
                }
                "schema_continuation_descriptor" => {
                    if schema_continuation_descriptor.is_some() {
                        return Err(M::Error::duplicate_field("schema_continuation_descriptor"));
                    }
                    schema_continuation_descriptor = Some(map.next_value()?);
                }
                "schema_reconciliation_descriptor" => {
                    if schema_reconciliation_descriptor.is_some() {
                        return Err(M::Error::duplicate_field(
                            "schema_reconciliation_descriptor",
                        ));
                    }
                    schema_reconciliation_descriptor = Some(map.next_value()?);
                }
                "descriptor_semantics_version" => {
                    if descriptor_semantics_version.is_some() {
                        return Err(M::Error::duplicate_field("descriptor_semantics_version"));
                    }
                    descriptor_semantics_version = Some(map.next_value()?);
                }
                _ => return Err(M::Error::custom("unknown canonical envelope field")),
            }
        }
        Ok(CanonicalEnvelopeWire {
            content: content.ok_or_else(|| M::Error::missing_field("content"))?,
            required_output_codec_retirements: required_output_codec_retirements
                .ok_or_else(|| M::Error::missing_field("required_output_codec_retirements"))?,
            required_output_origin: required_output_origin
                .ok_or_else(|| M::Error::missing_field("required_output_origin"))?,
            authored_cascade_retirement: authored_cascade_retirement
                .ok_or_else(|| M::Error::missing_field("authored_cascade_retirement"))?,
            authored_cascade_restoration: authored_cascade_restoration
                .ok_or_else(|| M::Error::missing_field("authored_cascade_restoration"))?,
            required_output_materialization: required_output_materialization
                .ok_or_else(|| M::Error::missing_field("required_output_materialization"))?,
            // The current canonical producer omits exactly the unavailable form.
            authored_index_writes: match authored_index_writes {
                Some(writes) => writes,
                None => super::super::CanonicalAuthoredIndexWrites::unavailable(),
            },
            commit: commit.ok_or_else(|| M::Error::missing_field("commit"))?,
            branch_context: branch_context
                .ok_or_else(|| M::Error::missing_field("branch_context"))?,
            branch_cell_checkpoint: branch_cell_checkpoint
                .ok_or_else(|| M::Error::missing_field("branch_cell_checkpoint"))?,
            authority_kind: authority_kind
                .ok_or_else(|| M::Error::missing_field("authority_kind"))?,
            strategy_artifacts: strategy_artifacts
                .ok_or_else(|| M::Error::missing_field("strategy_artifacts"))?,
            merge_execution_authority: merge_execution_authority
                .ok_or_else(|| M::Error::missing_field("merge_execution_authority"))?,
            merge_parent_branches: merge_parent_branches
                .ok_or_else(|| M::Error::missing_field("merge_parent_branches"))?,
            merge_base_commits: merge_base_commits
                .ok_or_else(|| M::Error::missing_field("merge_base_commits"))?,
            schema_version: schema_version
                .ok_or_else(|| M::Error::missing_field("schema_version"))?,
            schema_authority: schema_authority
                .ok_or_else(|| M::Error::missing_field("schema_authority"))?,
            merged_plan: merged_plan.ok_or_else(|| M::Error::missing_field("merged_plan"))?,
            symbol_texts: symbol_texts.ok_or_else(|| M::Error::missing_field("symbol_texts"))?,
            revision_symbol_references: revision_symbol_references
                .ok_or_else(|| M::Error::missing_field("revision_symbol_references"))?,
            record_allocations: record_allocations
                .ok_or_else(|| M::Error::missing_field("record_allocations"))?,
            patch: patch.ok_or_else(|| M::Error::missing_field("patch"))?,
            descriptive_touches: descriptive_touches
                .ok_or_else(|| M::Error::missing_field("descriptive_touches"))?,
            diagnostics_summary: diagnostics_summary
                .ok_or_else(|| M::Error::missing_field("diagnostics_summary"))?,
            lineage: lineage.ok_or_else(|| M::Error::missing_field("lineage"))?,
            derived_index_artifacts: derived_index_artifacts
                .ok_or_else(|| M::Error::missing_field("derived_index_artifacts"))?,
            schema_transition: schema_transition
                .ok_or_else(|| M::Error::missing_field("schema_transition"))?,
            schema_continuation_descriptor: schema_continuation_descriptor
                .ok_or_else(|| M::Error::missing_field("schema_continuation_descriptor"))?,
            schema_reconciliation_descriptor: schema_reconciliation_descriptor
                .ok_or_else(|| M::Error::missing_field("schema_reconciliation_descriptor"))?,
            descriptor_semantics_version: descriptor_semantics_version
                .ok_or_else(|| M::Error::missing_field("descriptor_semantics_version"))?,
        })
    }
}
