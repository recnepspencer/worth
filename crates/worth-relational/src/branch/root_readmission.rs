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

impl RelationalBranchRoot {
    pub(crate) fn readmit(
        issuer: &RelationalBranchRootIdentityIssuer,
        partitions: BTreeMap<PartitionId, PartitionState>,
        envelope: Arc<CanonicalCommitEnvelope>,
        descriptor: RelationalBranchRootDescriptor,
        schema_authority: Arc<super::RelationalBranchRootSchemaAuthority>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        validate_schema_root(&envelope, &descriptor)?;
        let root_id = issuer.issue_root_id()?;
        let entity_slot_count = partitions
            .values()
            .map(|partition| partition.entity_arena.slot_count())
            .sum();
        let relation_slot_count = partitions
            .values()
            .map(|partition| partition.relation_arena.slot_count())
            .sum();
        let (regions, mut publication_cost) =
            capture::build_initial_regions_owned(issuer, root_id, partitions, symbols)?;
        let (storage_root, content_accumulator) = axes::storage_root_for(&regions);
        require_descriptor_content(&descriptor, storage_root)?;
        let schema_root = *descriptor.schema_root();
        if !schema_authority.matches(&envelope.schema_authority)
            || schema_authority.descriptor_semantics_version()
                != envelope.descriptor_semantics_version
        {
            return Err(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
                descriptor_root: schema_root,
                canonical_root: schema_authority.registry().authority_digest_bytes(),
            });
        }
        publication_cost.new_schema_authorities = 1;
        publication_cost.new_authoritative_bytes = publication_cost
            .new_authoritative_bytes
            .saturating_add(schema_authority.authoritative_allocation_bytes());
        let correctness_index = RelationalRootCorrectnessIndex::AuthoritativeFallback;
        let axes = RelationalBranchRootAxes {
            storage_version: envelope.commit.version_id.0,
            storage_root,
            schema_root,
            correctness_index,
            visibility: RelationalBranchVisibilityCommitment::for_root(
                &envelope,
                storage_root,
                schema_root,
                correctness_index,
            ),
        };
        Ok(Arc::new(Self {
            id: root_id,
            regions,
            entity_slot_count,
            relation_slot_count,
            content_accumulator,
            schema_authority,
            committed: Some(RelationalCommittedBranchRoot {
                descriptor,
                canonical_envelope: envelope,
                axes,
                publication_cost,
            }),
        }))
    }

    /// Recovery can replace a reconstructed envelope with its durable
    /// canonical peer without rebuilding immutable storage regions.
    pub(crate) fn relink_canonical_envelope(
        &self,
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
        self.require_current_content_binding(&committed, symbols)?;
        require_visibility_commitment(&committed, &envelope)?;
        committed.canonical_envelope = envelope;
        Ok(self.with_committed(committed))
    }

    /// Readmit the durable descriptor learned from the next exact branch-cell
    /// checkpoint without rebuilding the reconstructed storage regions.
    pub(crate) fn readmit_descriptor(
        &self,
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
        self.require_current_content_binding(&committed, symbols)?;
        let (reconstructed_root, reconstructed_accumulator) =
            axes::storage_root_from_authoritative_regions(&self.regions, symbols)?;
        require_descriptor_content(&descriptor, reconstructed_root)?;
        debug_assert_eq!(self.content_accumulator, reconstructed_accumulator);
        committed.descriptor = descriptor;
        Ok(self.with_committed(committed))
    }

    fn require_current_content_binding(
        &self,
        committed: &RelationalCommittedBranchRoot,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), RelationalBranchRootCaptureDenial> {
        let (reconstructed_root, reconstructed_accumulator) =
            axes::storage_root_from_authoritative_regions(&self.regions, symbols)?;
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
            id: self.id,
            regions: Arc::clone(&self.regions),
            entity_slot_count: self.entity_slot_count,
            relation_slot_count: self.relation_slot_count,
            content_accumulator: self.content_accumulator,
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
