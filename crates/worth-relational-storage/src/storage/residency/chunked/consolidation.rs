//! History consolidation and cold verification of bound chunks.
//!
//! Consolidation flattens a chunk's layered history into a new member, which
//! changes `history_locator` but not the chunk's meaning. The consolidator
//! re-derives the semantic entry from the candidate members and installs the
//! new locator only when that entry is bit-identical to the sealed one, so
//! `root_semantic_merkle` (and every token bound to it) survives.
use super::canonical_read::{read_chunk_canonical, ChunkReadError, ChunkResidency, SlotSelection};
use super::layout::ChunkLayout;
use super::locator_binding::ChunkLocatorBinding;
use super::member_io::{ChunkMemberSink, ChunkMemberSource};
use super::semantic_entry::{ChunkCommitmentHasher, ChunkSemanticEntry};
use super::slot_range::SlotDomain;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkVerificationRefusal {
    /// The binding names another chunk than the entry.
    RangeMismatch,
    /// The members' derived meaning differs from the sealed entry: an edited
    /// member, an omitted edge or kind, or a non-equivalent consolidation.
    CommitmentDiverged {
        sealed: [u8; 32],
        derived: Box<ChunkSemanticEntry>,
    },
    Member(Cause),
}

impl From<ChunkReadError<std::convert::Infallible>> for ChunkVerificationRefusal {
    fn from(error: ChunkReadError<std::convert::Infallible>) -> Self {
        match error {
            ChunkReadError::Member(cause) => Self::Member(cause),
            ChunkReadError::Visitor(never) => match never {},
        }
    }
}

/// Re-derives the semantic entry of `domain`'s chunk from its bound members.
pub fn derive_bound_entry(
    residency: &ChunkResidency,
    source: &dyn ChunkMemberSource,
    domain: SlotDomain,
    binding: &ChunkLocatorBinding,
) -> Result<ChunkSemanticEntry, ChunkVerificationRefusal> {
    let mut hasher = ChunkCommitmentHasher::new(domain, binding.slot_range());
    read_chunk_canonical(
        residency,
        source,
        domain,
        binding,
        SlotSelection::All,
        &mut hasher,
    )?;
    Ok(hasher.finish().commit())
}

/// Proves that `binding`'s members carry exactly the meaning `sealed` commits.
pub fn verify_bound_chunk(
    residency: &ChunkResidency,
    source: &dyn ChunkMemberSource,
    sealed: &ChunkSemanticEntry,
    binding: &ChunkLocatorBinding,
) -> Result<(), ChunkVerificationRefusal> {
    if sealed.slot_range() != binding.slot_range() {
        return Err(ChunkVerificationRefusal::RangeMismatch);
    }
    let derived = derive_bound_entry(residency, source, sealed.domain(), binding)?;
    if derived != *sealed {
        return Err(ChunkVerificationRefusal::CommitmentDiverged {
            sealed: sealed.semantic_chunk_commitment(),
            derived: Box::new(derived),
        });
    }
    Ok(())
}

/// Writes the single-layer form of the bound chunk's history image and
/// returns its locator, the consolidation candidate.
pub fn flatten_history_image(
    residency: &ChunkResidency,
    source: &dyn ChunkMemberSource,
    sink: &mut dyn ChunkMemberSink,
    layout: ChunkLayout,
    binding: &ChunkLocatorBinding,
) -> Result<RelationalDurableMemberLocator, Cause> {
    let flattened = residency
        .history_image(source, binding.history_locator())?
        .flattened()
        .encode(layout.max_chunk_image_bytes)?;
    let locator = flattened.locator;
    sink.put(locator, flattened.bytes)?;
    Ok(locator)
}

/// A binding whose members were proven to carry exactly one sealed entry's
/// meaning. Only [`consolidate_history_chunk`] mints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsolidatedBinding {
    domain: SlotDomain,
    verified_commitment: [u8; 32],
    binding: ChunkLocatorBinding,
}

impl ConsolidatedBinding {
    pub fn binding(&self) -> &ChunkLocatorBinding {
        &self.binding
    }

    pub(super) fn into_parts(self) -> (SlotDomain, [u8; 32], ChunkLocatorBinding) {
        (self.domain, self.verified_commitment, self.binding)
    }
}

/// Rebinds the chunk's history to `candidate` after proving the chunk's
/// semantic commitment is unchanged. A diverging candidate is refused and the
/// existing binding remains authoritative.
pub fn consolidate_history_chunk(
    residency: &ChunkResidency,
    source: &dyn ChunkMemberSource,
    sealed: &ChunkSemanticEntry,
    binding: &ChunkLocatorBinding,
    candidate: RelationalDurableMemberLocator,
) -> Result<ConsolidatedBinding, ChunkVerificationRefusal> {
    let consolidated = binding.with_history_locator(candidate);
    verify_bound_chunk(residency, source, sealed, &consolidated)?;
    Ok(ConsolidatedBinding {
        domain: sealed.domain(),
        verified_commitment: sealed.semantic_chunk_commitment(),
        binding: consolidated,
    })
}
