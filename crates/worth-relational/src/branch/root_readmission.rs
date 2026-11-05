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
        if committed.evidence.original().is_none()
            || committed.evidence.receipt() != &envelope.commit
        {
            return Err(RelationalBranchRootCaptureDenial::CanonicalEnvelopeMismatch);
        }
        validate_schema_root(&envelope, &committed.descriptor)?;
        require_schema_authority(&self.schema_authority, &envelope, &committed.descriptor)?;
        self.require_current_content_binding(&committed, symbols)?;
        require_visibility_commitment(&committed, &envelope)?;
        self.content
            .certify_records(
                execution,
                self.schema_authority.registry(),
                envelope.content.records(),
            )
            .map_err(RelationalBranchRootCaptureDenial::Content)?;
        committed.evidence = super::RelationalRootCommitEvidence::Original(envelope);
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
        self.require_current_content_binding(&committed, symbols)?;
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
        committed.evidence =
            super::RelationalRootCommitEvidence::Original(source.canonical_arc().clone());
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
        if !committed.evidence.matches_schema(&self.schema_authority)
            || descriptor.schema_root() != &self.schema_authority.authority_digest()
        {
            return Err(RelationalBranchRootCaptureDenial::SchemaRootMismatch {
                descriptor_root: *descriptor.schema_root(),
                canonical_root: self.schema_authority.authority_digest(),
            });
        }
        self.require_current_content_binding(&committed, symbols)?;
        let (reconstructed_root, reconstructed_accumulator) =
            axes::storage_root_from_authoritative_regions(&self.regions, symbols)?;
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
        require_descriptor_content(&committed.descriptor, reconstructed_root)?;
        match &committed.evidence {
            super::RelationalRootCommitEvidence::Original(envelope) => {
                require_visibility_commitment(committed, envelope)
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
            output_publication_positions: match &committed.evidence {
                super::RelationalRootCommitEvidence::Original(envelope) => {
                    crate::history::data::RequiredOutputPublicationPositions::unresolved_at(
                        envelope.commit.commit_id,
                    )
                }
                super::RelationalRootCommitEvidence::Checkpoint(_) => {
                    self.output_publication_positions.clone()
                }
            },
            schema_authority: Arc::clone(&self.schema_authority),
            committed: Some(committed),
            _memory: None,
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
