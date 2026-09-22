use crate::commit_strategies::data::StrategyReplayDescriptor;
use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::history::data::{BranchId, CommitId, OrderedParentList, RelationalCommitReceipt};
use crate::indexes::data::{DerivedIndexArtifacts, DerivedIndexEntries};
#[cfg(test)]
use crate::publication::patch::data::PublishedAuthoritativePatchEnvelope;
use crate::replay::data::ReplaySnapshotSurface;

use super::primitive_terms::ReplayDigestBuilder;

#[cfg(test)]
pub(crate) fn digest_patch_surface(patch: &PublishedAuthoritativePatchEnvelope) -> [u8; 32] {
    let canonical = patch.canonicalized();
    let mut builder = ReplayDigestBuilder::new("worth.relational.replay.surface.patch.v1")
        .patch_ordering(canonical.ordering)
        .patch_publication_mode(canonical.publication_mode)
        .patch_stream_position(canonical.position)
        .usize(canonical.authoritative_record_patches.len());
    for record in &canonical.authoritative_record_patches {
        builder = builder
            .record_ref(&record.target)
            .structural_change(record.structural_change)
            .published_patch(&record.authoritative_patch)
            .bool(record.contains_opaque_aspect)
            .patch_detail(&record.detail);
    }
    builder.finish()
}

pub(crate) fn digest_canonical_patch_surface(
    patch: &crate::publication::patch::data::CanonicalAuthoritativePatch,
) -> [u8; 32] {
    let canonical = patch.canonicalized();
    let mut builder =
        ReplayDigestBuilder::new("worth.relational.replay.surface.canonical-patch.v1")
            .patch_ordering(canonical.ordering)
            .patch_publication_mode(canonical.publication_mode)
            .usize(canonical.authoritative_record_patches.len());
    for record in &canonical.authoritative_record_patches {
        builder = builder
            .record_ref(&record.target)
            .structural_change(record.structural_change)
            .published_patch(&record.authoritative_patch)
            .bool(record.contains_opaque_aspect)
            .patch_detail(&record.detail);
    }
    builder.finish()
}

pub(crate) fn digest_canonical_patch_summary(
    patch: &crate::publication::patch::data::CanonicalAuthoritativePatch,
) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.canonical-patch.v1")
        .usize(patch.authoritative_record_patches.len())
        .finish()
}

pub(crate) fn digest_diagnostics_surface(summary: &RelationalDiagnosticArtifact) -> [u8; 32] {
    let mut builder = ReplayDigestBuilder::new("worth.relational.replay.surface.diagnostics.v1")
        .label(summary.scope)
        .label(summary.kind)
        .label(summary.determinism)
        .usize(summary.entries.len());
    for entry in &summary.entries {
        builder = builder
            .label(entry.code)
            .string(&entry.message)
            .diagnostic_value(entry.fields.root());
    }
    builder.finish()
}

pub(crate) fn digest_diagnostics_summary(summary: &RelationalDiagnosticArtifact) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.diagnostics.v1")
        .usize(summary.entries.len())
        .label(summary.kind)
        .finish()
}

pub(crate) fn digest_history_surface(
    parents: &OrderedParentList,
    merge_parent_branches: &[BranchId],
    merge_base_commits: &[CommitId],
) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.surface.history.v1")
        .ordered_parent_list(parents)
        .branch_id_sequence(merge_parent_branches)
        .commit_id_sequence(merge_base_commits)
        .finish()
}

pub(crate) fn digest_history_summary(parents: &OrderedParentList) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.history.v1")
        .usize(parents.len())
        .finish()
}

pub(crate) fn digest_snapshot_surface(surface: &ReplaySnapshotSurface) -> [u8; 32] {
    let mut builder = ReplayDigestBuilder::new("worth.relational.replay.surface.snapshot.v1")
        .version_id(surface.version_id)
        .usize(surface.entities.len())
        .usize(surface.relations.len());
    for entity in &surface.entities {
        builder = builder.record_ref(&crate::transactions::data::RecordRef::Entity(
            entity.entity_id,
        ));
        if let Some(state) = entity.authoritative_aspect_state.as_ref() {
            for (aspect_key, value) in state.aspects().entries() {
                builder = builder
                    .aspect_key(aspect_key)
                    .validated_value_for_surface(value);
            }
        }
    }
    for relation in &surface.relations {
        builder = builder.record_ref(&crate::transactions::data::RecordRef::Relation(
            relation.relation_id,
        ));
        if let Some(state) = relation.authoritative_aspect_state.as_ref() {
            for (aspect_key, value) in state.aspects().entries() {
                builder = builder
                    .aspect_key(aspect_key)
                    .validated_value_for_surface(value);
            }
        }
    }
    builder.finish()
}

pub(crate) fn digest_snapshot_summary(surface: &ReplaySnapshotSurface) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.snapshot.v1")
        .version_id(surface.version_id)
        .usize(surface.entities.len())
        .usize(surface.relations.len())
        .finish()
}

pub(crate) fn digest_branch_head_surface(commit: Option<&RelationalCommitReceipt>) -> [u8; 32] {
    let builder = ReplayDigestBuilder::new("worth.relational.replay.surface.branch_head.v1");
    match commit {
        Some(commit) => builder
            .tag(1)
            .commit_id(commit.commit_id)
            .version_id(commit.version_id)
            .branch_id(&commit.branch_id)
            .commit_id_sequence(&commit.parents)
            .finish(),
        None => builder.tag(0).finish(),
    }
}

pub(crate) fn digest_branch_head_summary(commit: Option<&RelationalCommitReceipt>) -> [u8; 32] {
    let builder = ReplayDigestBuilder::new("worth.relational.replay.summary.branch_head.v1");
    match commit {
        Some(commit) => builder
            .tag(1)
            .commit_id(commit.commit_id)
            .version_id(commit.version_id)
            .finish(),
        None => builder.tag(0).finish(),
    }
}

pub(crate) fn digest_strategy_replay_descriptor(descriptor: &StrategyReplayDescriptor) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.surface.strategy_descriptor.v1")
        .u64(descriptor.strategy_id().0 as u64)
        .digest_bytes(&descriptor.descriptor_digest().0)
        .digest_bytes(&descriptor.input_digest().0)
        .digest_bytes(&descriptor.output_digest().0)
        .digest_bytes(&descriptor.mutation_program_digest().0)
        .digest_bytes(descriptor.lowering_summary_digest())
        .optional_digest(descriptor.preview_validation_summary_digest())
        .optional_digest(descriptor.preview_validation_cost_digest())
        .optional_commit_id(descriptor.validated_against_commit_id())
        .optional_version_id(descriptor.validated_against_version_id())
        .finish()
}

pub(crate) fn digest_strategy_replay_summary(descriptor: &StrategyReplayDescriptor) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.strategy_descriptor.v1")
        .u64(descriptor.strategy_id().0 as u64)
        .digest_bytes(&descriptor.input_digest().0)
        .digest_bytes(&descriptor.output_digest().0)
        .digest_bytes(&descriptor.mutation_program_digest().0)
        .optional_commit_id(descriptor.validated_against_commit_id())
        .optional_version_id(descriptor.validated_against_version_id())
        .finish()
}

pub(crate) fn digest_derived_index_surface(artifacts: &DerivedIndexArtifacts) -> [u8; 32] {
    let mut builder = ReplayDigestBuilder::new("worth.relational.replay.surface.derived_index.v1")
        .usize(artifacts.generations().len());
    for generation in artifacts.generations() {
        builder = builder
            .u64(generation.generation_id.0)
            .u64(generation.index_id.0)
            .commit_id(generation.source_commit_id)
            .branch_id(&generation.source_branch_id)
            .branch_id(&generation.applicability.branch_id)
            .version_id(generation.applicability.version_id)
            .schema_version_id(generation.applicability.schema_version)
            .label(generation.status)
            .derived_index_entries(&generation.entries);
    }
    builder.finish()
}

pub(crate) fn digest_derived_index_summary(artifacts: &DerivedIndexArtifacts) -> [u8; 32] {
    ReplayDigestBuilder::new("worth.relational.replay.summary.derived_index.v1")
        .usize(artifacts.generations().len())
        .finish()
}

trait ReplaySurfaceDigestBuilderExt {
    fn validated_value_for_surface(
        self,
        value: &worth_foundational::facade::ContractValidatedAspectValue,
    ) -> Self;
    fn derived_index_entries(self, entries: &DerivedIndexEntries) -> Self;
}

impl ReplaySurfaceDigestBuilderExt for ReplayDigestBuilder {
    fn validated_value_for_surface(
        self,
        value: &worth_foundational::facade::ContractValidatedAspectValue,
    ) -> Self {
        let contract_revision = value.contract_revision().0;
        match value.view() {
            worth_foundational::facade::ContractValidatedAspectValueView::Scalar(value) => {
                self.tag(1).u64(contract_revision).aspect_value(value)
            }
            worth_foundational::facade::ContractValidatedAspectValueView::Struct(value) => {
                self.tag(2).u64(contract_revision).struct_value(value)
            }
        }
    }

    fn derived_index_entries(mut self, entries: &DerivedIndexEntries) -> Self {
        match entries {
            DerivedIndexEntries::EntityField(rows) => {
                self = self.tag(1).usize(rows.len());
                for (key, ids) in rows.iter() {
                    self = self.byte_vec(key.canonical_value_bytes()).usize(ids.len());
                    for id in ids {
                        self = self.entity_id(*id);
                    }
                }
                self
            }
            DerivedIndexEntries::RelationField(rows) => {
                self = self.tag(2).usize(rows.len());
                for (key, ids) in rows.iter() {
                    self = self.byte_vec(key.canonical_value_bytes()).usize(ids.len());
                    for id in ids {
                        self = self.relation_id(*id);
                    }
                }
                self
            }
            DerivedIndexEntries::RelatedEntityOrdering(rows) => {
                self = self.tag(3).usize(rows.len());
                for (parent, entries) in rows.iter() {
                    self = self.entity_id(*parent).usize(entries.len());
                    for entry in entries {
                        self = self.usize(entry.ordering_values().len());
                        for value in entry.ordering_values() {
                            self = self.aspect_value(value.value());
                        }
                        self = self
                            .entity_id(entry.child_entity_id())
                            .relation_id(entry.relation_id());
                    }
                }
                self
            }
            DerivedIndexEntries::RelationJoin(rows) => {
                self = self.tag(4).usize(rows.len());
                for (key, entries) in rows.iter() {
                    self = self
                        .entity_id(key.left_entity_id())
                        .entity_id(key.right_entity_id())
                        .usize(entries.len());
                    for entry in entries {
                        self = self
                            .entity_id(entry.shared_entity_id())
                            .relation_id(entry.left_relation_id())
                            .relation_id(entry.right_relation_id());
                    }
                }
                self
            }
        }
    }
}
