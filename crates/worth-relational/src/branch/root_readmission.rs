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

#[path = "root_readmission/checkpoint.rs"]
mod checkpoint;
#[path = "root_readmission/request.rs"]
mod request;
#[path = "root_readmission/required_text.rs"]
mod required_text;

#[path = "root_readmission/region_binding.rs"]
mod region_binding;
use region_binding::ReadmittedRegionSet;

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
        original: Arc<super::RelationalOriginalRootCommitEvidence>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let envelope = Arc::clone(original.envelope());
        let mut committed = self
            .committed
            .clone()
            .ok_or(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch)?;
        if committed.evidence.original().is_none()
            || committed.evidence.receipt() != &envelope.commit
        {
            return Err(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch);
        }
        validate_schema_root(&envelope, &committed.descriptor)?;
        require_schema_authority(&self.schema_authority, &envelope, &committed.descriptor)?;
        self.require_current_content_binding(execution, &committed, symbols)?;
        self.content
            .certify_records(
                execution,
                self.schema_authority.registry(),
                envelope.content.records(),
            )
            .map_err(RelationalBranchRootCaptureDenial::Content)?;
        // Readmission replaces the envelope owner, so the old change proof is discarded.
        committed.commit_change = None;
        let evidence = super::RelationalRootCommitEvidence::Original(Arc::clone(&original));
        let visibility = RelationalBranchVisibilityCommitment::for_payload_for_request(
            original.payload(),
            committed.axes.storage_root,
            committed.axes.schema_root,
            committed.axes.correctness_index,
            execution,
        )?;
        if visibility != committed.axes.visibility {
            return Err(
                RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch {
                    committed: committed.axes.visibility.digest(),
                    reconstructed: visibility.digest(),
                },
            );
        }
        committed.evidence = evidence;
        self.with_committed(execution, committed)
    }

    /// Recovery binds admitted source authority after replay reconstructs storage.
    /// Its visibility axis includes retained strategy facts absent from execution.
    pub(crate) fn readmit_canonical_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &crate::history::data::PositionedCanonicalCommit,
        original: Arc<super::RelationalOriginalRootCommitEvidence>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        if !Arc::ptr_eq(original.envelope(), source.canonical_arc()) {
            return Err(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch);
        }
        let mut committed = self
            .committed
            .clone()
            .ok_or(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch)?;
        if committed.evidence.original().is_none() || committed.evidence.receipt() != &source.commit
        {
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
        let evidence = super::RelationalRootCommitEvidence::Original(Arc::clone(&original));
        committed.axes.visibility = RelationalBranchVisibilityCommitment::for_payload_for_request(
            original.payload(),
            committed.axes.storage_root,
            committed.axes.schema_root,
            committed.axes.correctness_index,
            execution,
        )?;
        committed.commit_change = None;
        committed.evidence = evidence;
        self.with_committed(execution, committed)
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
        if !committed.evidence.matches_schema(&self.schema_authority)
            || descriptor.schema_root() != &self.schema_authority.authority_digest()
        {
            return Err(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
                descriptor_root: *descriptor.schema_root(),
                canonical_root: self.schema_authority.authority_digest(),
            });
        }
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
        self.with_committed(execution, committed)
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
        require_descriptor_content(&committed.descriptor, reconstructed_root)?;
        match &committed.evidence {
            super::RelationalRootCommitEvidence::Original(original) => {
                require_visibility_commitment(committed, original.envelope())
            }
            super::RelationalRootCommitEvidence::Checkpoint(commit) => {
                if committed.axes.visibility != commit.visibility() {
                    return Err(
                        RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch {
                            committed: committed.axes.visibility.digest(),
                            reconstructed: commit.visibility().digest(),
                        },
                    );
                }
                Ok(())
            }
        }
    }

    fn with_committed(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        committed: RelationalCommittedBranchRoot,
    ) -> Result<Arc<Self>, RelationalBranchRootCaptureDenial> {
        let memory = super::reserve_arc_memory::<Self>(Some(execution))?;
        Ok(Arc::new(Self {
            publication_bases: Default::default(),
            content: self.content.clone(),
            // The callers preserve this exact root, schema and content binding.
            managed_source_image: self.managed_source_image.clone(),
            id: self.id,
            regions: Arc::clone(&self.regions),
            entity_slot_count: self.entity_slot_count,
            relation_slot_count: self.relation_slot_count,
            content_accumulator: self.content_accumulator,
            output_publication_positions: match &committed.evidence {
                super::RelationalRootCommitEvidence::Original(original) => {
                    crate::history::data::RequiredOutputPublicationPositions::unresolved_at(
                        original.envelope().commit.commit_id,
                    )
                }
                super::RelationalRootCommitEvidence::Checkpoint(_) => {
                    self.output_publication_positions.clone()
                }
            },
            execution_contracts: self.execution_contracts.clone(),
            schema_authority: Arc::clone(&self.schema_authority),
            committed: Some(committed),
            _memory: memory,
        }))
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
            canonical_root: authority.authority_digest(),
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
