//! Proof that a chunk is the occurrence of a source generation that a reuse
//! claim or a dedupe quarantine names.

use super::super::blob_record::{BlobFact, ReuseSourceWitness};
use super::proof::{needed_row, Proof};
use super::row_index::RowIndex;
use super::Selected;

/// The source occurrence a reuse claim or a dedupe quarantine names.
pub(super) struct SourceClaim {
    pub(super) publication: [u8; 24],
    pub(super) ordinal: u64,
    pub(super) chunk: [u8; 24],
    pub(super) digest: [u8; 32],
    /// A quarantine states no length; the source generation decides it.
    pub(super) length: Option<u64>,
    pub(super) scope: [u8; 32],
    pub(super) chunk_size: u32,
}

/// The generation a source occurrence belongs to, as its selected publication
/// states it or as a claim witnessed it.
struct SourceGeneration {
    store: [u8; 16],
    session: [u8; 16],
    root: [u8; 24],
    root_digest: [u8; 32],
    total: u64,
    chunk_size: u32,
    scope: [u8; 32],
}

impl SourceGeneration {
    fn published(fact: &BlobFact) -> Option<Self> {
        let BlobFact::Publication {
            store,
            session,
            root,
            root_digest,
            total,
            chunk_size,
            scope,
            ..
        } = fact
        else {
            return None;
        };
        Some(Self {
            store: *store,
            session: *session,
            root: *root,
            root_digest: *root_digest,
            total: *total,
            chunk_size: *chunk_size,
            scope: *scope,
        })
    }

    fn witnessed(witness: &ReuseSourceWitness) -> Self {
        Self {
            store: witness.store,
            session: witness.session,
            root: witness.root,
            root_digest: witness.root_digest,
            total: witness.total,
            chunk_size: witness.chunk_size,
            scope: witness.scope,
        }
    }
}

impl SourceClaim {
    /// The length of the claimed occurrence in `source`, or `None` when the
    /// claim cannot name an occurrence of that generation.
    fn length_in(&self, source: &SourceGeneration) -> Option<u64> {
        let size = u64::from(self.chunk_size);
        let offset = self.ordinal.checked_mul(size)?;
        let length = source
            .total
            .checked_sub(offset)
            .filter(|remaining| *remaining > 0)?
            .min(size);
        (source.scope == self.scope
            && source.chunk_size == self.chunk_size
            && self.length.is_none_or(|stated| stated == length))
        .then_some(length)
    }

    /// The chunk `source` owns at the claimed ordinal.
    fn occurrence(&self, source: &SourceGeneration) -> Option<BlobFact> {
        Some(BlobFact::Chunk {
            store: source.store,
            session: source.session,
            ordinal: self.ordinal,
            chunk_size: self.chunk_size,
            length: self.length_in(source)?,
            digest: self.digest,
        })
    }

    /// What the claim alone requires of the chunk it borrows, whoever owns it.
    fn names(&self, chunk: &BlobFact) -> bool {
        let BlobFact::Chunk {
            ordinal,
            chunk_size,
            length,
            digest,
            ..
        } = chunk
        else {
            return false;
        };
        *ordinal == self.ordinal
            && *chunk_size == self.chunk_size
            && *digest == self.digest
            && self.length.is_none_or(|stated| stated == *length)
    }
}

/// The claim against its selected source publication: the publication's own
/// declaration, the tree edges down to the leaf edge that names the borrowed
/// chunk, and the chunk itself. `witness` is a claim's own copy of the
/// publication; it says whose chunk is borrowed when the publication's frame
/// could not be read.
pub(super) fn proof(
    selected: &[Selected],
    rows: &RowIndex,
    claim: &SourceClaim,
    witness: Option<&ReuseSourceWitness>,
) -> Proof {
    let publication = match needed_row(selected, rows.record(&claim.publication)) {
        Ok(publication) => publication,
        Err(verdict) => return chunk_unpublished(selected, rows, claim, witness).and(verdict),
    };
    let Some(source) = SourceGeneration::published(publication.fact) else {
        return Proof::Contradicted;
    };
    let Some(occurrence) = claim.occurrence(&source) else {
        return Proof::Contradicted;
    };
    let declared = Proof::on_row(selected, rows.declaration(&source.session), |fact| {
        matches!(fact, BlobFact::Declaration { store, scope, chunk_size, total, .. }
            if *store == source.store
                && *scope == source.scope
                && *chunk_size == source.chunk_size
                && *total == source.total)
    });
    borrowed_chunk(selected, rows, claim, Some(&occurrence))
        .and(declared)
        .and(leaf_edge(selected, rows, claim, &source, &occurrence))
        .and(publication.standing)
}

/// The borrowed chunk when no publication frame says whose it is, because the
/// publication could not be read or was released: against the witnessed
/// generation if the claim carries one, else against what the claim alone
/// states. The chunk must be selected either way. A release drops only what
/// nothing selected protects, and a selected claim protects the chunk it
/// borrows; the Store opens that chunk for every selected claim.
///
/// This is not the tree proof: the leaf edge, the root digest and the source
/// declaration are not checked against the witness.
pub(super) fn chunk_unpublished(
    selected: &[Selected],
    rows: &RowIndex,
    claim: &SourceClaim,
    witness: Option<&ReuseSourceWitness>,
) -> Proof {
    let Some(source) = witness.map(SourceGeneration::witnessed) else {
        return borrowed_chunk(selected, rows, claim, None);
    };
    match claim.occurrence(&source) {
        Some(occurrence) => borrowed_chunk(selected, rows, claim, Some(&occurrence)),
        None => Proof::Contradicted,
    }
}

fn borrowed_chunk(
    selected: &[Selected],
    rows: &RowIndex,
    claim: &SourceClaim,
    occurrence: Option<&BlobFact>,
) -> Proof {
    Proof::on_row(selected, rows.record(&claim.chunk), |chunk| {
        occurrence.map_or_else(|| claim.names(chunk), |owned| owned == chunk)
    })
}

/// The tree edges from the generation's root down to the leaf edge that names
/// the borrowed chunk.
fn leaf_edge(
    selected: &[Selected],
    rows: &RowIndex,
    claim: &SourceClaim,
    source: &SourceGeneration,
    occurrence: &BlobFact,
) -> Proof {
    let BlobFact::Chunk { length, .. } = occurrence else {
        return Proof::Contradicted;
    };
    let Some(mut offset) = claim.ordinal.checked_mul(u64::from(claim.chunk_size)) else {
        return Proof::Contradicted;
    };
    let mut node_record = source.root;
    let mut expected_digest = source.root_digest;
    let mut standing = Proof::Holds;
    for depth in 0..7 {
        let node = match needed_row(selected, rows.record(&node_record)) {
            Ok(node) => node,
            Err(verdict) => return standing.and(verdict),
        };
        standing = standing.and(node.standing);
        let BlobFact::Node {
            session,
            kind,
            covered,
            digest,
            frame_digest,
            entries,
            ..
        } = node.fact
        else {
            return Proof::Contradicted;
        };
        let named = if depth == 0 { frame_digest } else { digest };
        if *session != source.session || *named != expected_digest || offset >= *covered {
            return Proof::Contradicted;
        }
        let mut selected_edge = None;
        for edge in entries {
            if offset < edge.covered {
                selected_edge = Some(edge);
                break;
            }
            offset -= edge.covered;
        }
        let Some(edge) = selected_edge else {
            return Proof::Contradicted;
        };
        if *kind == 1 {
            return Proof::of(
                edge.record == claim.chunk
                    && edge.digest == claim.digest
                    && edge.covered == *length
                    && offset == 0,
            )
            .and(standing);
        }
        if *kind != 2 {
            return Proof::Contradicted;
        }
        node_record = edge.record;
        expected_digest = edge.digest;
    }
    Proof::Contradicted
}
