use std::collections::BTreeMap;
use std::sync::Arc;

use crate::history::data::CanonicalCommitEnvelope;
use crate::identity::data::PartitionId;
use crate::storage::overlay::PartitionState;

use super::{
    axes, capture, RelationalBranchRoot, RelationalBranchRootAxes,
    RelationalBranchRootCaptureDenial, RelationalBranchRootDescriptor,
    RelationalBranchRootIdentityIssuer, RelationalBranchVisibilityCommitment,
    RelationalCommittedBranchRoot, RelationalRootCorrectnessIndex,
};

#[path = "root_readmission/required_text.rs"]
mod required_text;

#[path = "root_readmission/region_binding.rs"]
mod region_binding;

impl RelationalBranchRoot {
    /// Closed corruption recorders use the same content owner as publication.
    pub(crate) fn reconstruct_storage_root_for_test(
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partitions: BTreeMap<PartitionId, PartitionState>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<[u8; 32], RelationalBranchRootCaptureDenial> {
        let issuer = RelationalBranchRootIdentityIssuer::default();
        let (regions, _) =
            capture::build_initial_regions_owned(lane, execution, &issuer, 0, partitions, symbols)?;
        Ok(axes::storage_root_for(&regions).0)
    }
    /// Recovery can replace a reconstructed envelope with its durable
    /// canonical peer without rebuilding immutable storage regions.
    pub(crate) fn relink_canonical_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        envelope: Arc<CanonicalCommitEnvelope>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let mut committed = self
            .committed
            .clone()
            .ok_or(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch)?;
        if committed.canonical_envelope.commit != envelope.commit {
            return Err(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch);
        }
        validate_schema_root(&envelope, &committed.descriptor)?;
        require_schema_authority(&self.schema_authority, &envelope, &committed.descriptor)?;
        self.require_current_content_binding(execution, &committed, symbols)?;
        require_visibility_commitment(&committed, &envelope)?;
        self.content
            .certify_records(
                execution,
                self.schema_authority.registry(),
                envelope.content.records(),
            )
            .map_err(RelationalBranchRootCaptureDenial::Content)?;
        committed.canonical_envelope = envelope;
        Ok(self.with_committed(committed))
    }

    /// Recovery binds admitted source authority after replay reconstructs storage.
    /// Its visibility axis includes retained strategy facts absent from execution.
    pub(crate) fn readmit_canonical_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::history::data::PositionedCanonicalCommit,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let mut committed = self
            .committed
            .clone()
            .ok_or(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch)?;
        if committed.canonical_envelope.commit != source.commit {
            return Err(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch);
        }
        validate_schema_root(source.envelope(), &committed.descriptor)?;
        require_schema_authority(
            &self.schema_authority,
            source.envelope(),
            &committed.descriptor,
        )?;
        self.require_current_content_binding(execution, &committed, symbols)?;
        self.content
            .certify_records(
                execution,
                self.schema_authority.registry(),
                source.envelope().content.records(),
            )
            .map_err(RelationalBranchRootCaptureDenial::Content)?;
        committed.axes.visibility = RelationalBranchVisibilityCommitment::for_root(
            source.envelope(),
            committed.axes.storage_root,
            committed.axes.schema_root,
            committed.axes.correctness_index,
        );
        committed.canonical_envelope = source.canonical_arc().clone();
        Ok(self.with_committed(committed))
    }

    /// Readmit the durable descriptor learned from the next exact branch-cell
    /// checkpoint without rebuilding the reconstructed storage regions.
    pub(crate) fn readmit_descriptor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        descriptor: RelationalBranchRootDescriptor,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let mut committed = self
            .committed
            .clone()
            .ok_or(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch)?;
        validate_schema_root(&committed.canonical_envelope, &descriptor)?;
        require_schema_authority(
            &self.schema_authority,
            &committed.canonical_envelope,
            &descriptor,
        )?;
        self.require_current_content_binding(execution, &committed, symbols)?;
        let (reconstructed_root, reconstructed_accumulator) =
            self.verify_resident_storage_content(execution, symbols)?;
        require_descriptor_content(&descriptor, reconstructed_root)?;
        debug_assert_eq!(
            /* Proof: readmit rebuilds both the region set and accumulator from the same checked immutable regions before installing the new root; no caller can set the private accumulator independently. */
            self.content_accumulator,
            reconstructed_accumulator
        );
        committed.descriptor = descriptor;
        Ok(self.with_committed(committed))
    }

    fn require_current_content_binding(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        committed: &RelationalCommittedBranchRoot,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), RelationalBranchRootCaptureDenial> {
        let (reconstructed_root, reconstructed_accumulator) =
            self.verify_resident_storage_content(execution, symbols)?;
        if self.content_accumulator != reconstructed_accumulator
            || committed.axes.storage_root != reconstructed_root
        {
            return Err(RelationalBranchRootCaptureDenial::StorageContentMismatch {
                descriptor_root: *committed.descriptor.truth_root(),
                reconstructed_root,
            });
        }
        require_descriptor_content(&committed.descriptor, reconstructed_root)
            .and_then(|()| require_visibility_commitment(committed, &committed.canonical_envelope))
    }

    fn with_committed(&self, committed: RelationalCommittedBranchRoot) -> Arc<Self> {
        Arc::new(Self {
            content: self.content.clone(),
            // The callers preserve this exact root, schema and content binding.
            managed_source_image: self.managed_source_image.clone(),
            id: self.id,
            regions: Arc::clone(&self.regions),
            entity_slot_count: self.entity_slot_count,
            relation_slot_count: self.relation_slot_count,
            content_accumulator: self.content_accumulator,
            output_publication_positions:
                crate::history::data::RequiredOutputPublicationPositions::unresolved_at(
                    committed.canonical_envelope.commit.commit_id,
                ),
            schema_authority: Arc::clone(&self.schema_authority),
            committed: Some(committed),
        })
    }
}

fn require_schema_authority(
    authority: &super::RelationalBranchRootSchemaAuthority,
    envelope: &CanonicalCommitEnvelope,
    descriptor: &RelationalBranchRootDescriptor,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    if !authority.matches(&envelope.schema_authority)
        || authority.descriptor_semantics_version() != envelope.descriptor_semantics_version
    {
        return Err(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
            descriptor_root: *descriptor.schema_root(),
            canonical_root: authority.registry().authority_digest_bytes(),
        });
    }
    Ok(())
}

fn validate_schema_root(
    envelope: &CanonicalCommitEnvelope,
    descriptor: &RelationalBranchRootDescriptor,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    let canonical_root =
        crate::schema::data::schema_authority_snapshot_digest_bytes(&envelope.schema_authority);
    if descriptor.schema_root() != &canonical_root {
        return Err(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
            descriptor_root: *descriptor.schema_root(),
            canonical_root,
        });
    }
    Ok(())
}

fn require_visibility_commitment(
    committed: &RelationalCommittedBranchRoot,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), RelationalBranchRootCaptureDenial> {
    let reconstructed = RelationalBranchVisibilityCommitment::for_root(
        envelope,
        committed.axes.storage_root,
        committed.axes.schema_root,
        committed.axes.correctness_index,
    );
    if committed.axes.visibility != reconstructed {
        return Err(
            RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch {
                committed: committed.axes.visibility.digest(),
                reconstructed: reconstructed.digest(),
            },
        );
    }
    Ok(())
}

fn require_descriptor_content(
    descriptor: &RelationalBranchRootDescriptor,
    reconstructed_root: [u8; 32],
) -> Result<(), RelationalBranchRootCaptureDenial> {
    if descriptor.truth_root() != &reconstructed_root {
        return Err(RelationalBranchRootCaptureDenial::StorageContentMismatch {
            descriptor_root: *descriptor.truth_root(),
            reconstructed_root,
        });
    }
    Ok(())
}
