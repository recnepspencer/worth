//! The identity non-reissue fact for one terminal head retired.
//!
//! Invariant: when this fact is issued no publication of the object identity
//! or of its session is selected, and no ingest session can append one while
//! the fact lives or after it. The scan that binds the declaration finds no
//! selected publication, and three owners close the three ways to an append:
//!
//! - Begin. A selected SessionDeclared record is never unrouted: failed-ingest
//!   drop admission rejects a drop set naming its declaration, the extent
//!   resolution every reclaim shares rejects any drop set naming a
//!   declaration record, and abort and expiry append SessionAbandoned.
//!   Declaration admission therefore denies this object and this session
//!   forever.
//! - Resume. While a release head of the identity stands, the head keeps its
//!   release manifest selected, and that manifest denies resume and
//!   abandonment of the session. The completed durable checkpoint sequence is
//!   also strictly above the declaration's maximum. That sequence is durable
//!   and monotonic, so once the head is retired resume admission answers
//!   Expired in this process and after every reopen.
//! - Finish. The fact owns the session's one runtime claim, taken from the
//!   registry every begun or resumed session holds its live claim in. No live
//!   session exists to finish while the fact lives, and none can go live
//!   after it.
//!
//! The retirement record binds the declaration found here so C.9, planning
//! and rejoin can verify it again.

use std::num::NonZeroU64;
use std::sync::MutexGuard;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobSessionDeclarationV1, PersistedRecordIdentity, ReleaseCustodyHeadKeyV1,
    ReleasedGenerationReclaimBasisV1, RootPublicationCell,
};

use super::super::super::read::{
    selected_generation_publication, selected_session_declaration, walk_selected_reader,
    BlobReadOpenFailure,
};
use crate::physical_runtime::{
    durability::{PhysicalBlobSessionClaim, PhysicalBlobSessionClaimDenial},
    BlobSessionId, PhysicalRecordReader, ServingPhysicalRuntime,
};

#[derive(Debug)]
pub(in crate::physical_runtime) enum TerminalHeadNonReissueDenial {
    InvalidKey,
    /// Another holder of the session identity exists: a resume, terminal or
    /// reclaim attempt of the same session.
    Claim(PhysicalBlobSessionClaimDenial),
    Inspection(BlobReadOpenFailure),
    DeclarationAbsent,
    DeclarationConflict,
    /// A publication of the object or of its session is selected: the
    /// generation was never released, or its identity published again. No
    /// head of it may retire.
    PublicationSelected,
    /// Once the head is retired only expiry denies resume, and no completed
    /// checkpoint has passed the declaration's maximum yet, so the tombstone
    /// stays.
    DeclarationNotExpired {
        selected_checkpoint_sequence: u64,
        maximum_checkpoint_sequence: u64,
    },
}

/// Sealed: issued only by the declaration owner, which keeps its declaration
/// lock and the session's one runtime claim inside the fact. The selected
/// root still routes the exact SessionDeclared record that claimed this
/// object identity, that declaration is durably expired, and the root routes
/// no publication of the object or of its session.
pub(in crate::physical_runtime) struct TerminalHeadIdentityNonReissue<'runtime> {
    _declarations: MutexGuard<'runtime, ()>,
    session_claim: PhysicalBlobSessionClaim,
    key: ReleaseCustodyHeadKeyV1,
    root: RootPublicationCell,
    declaration_record: PersistedRecordIdentity,
    declaration_frame_sha256: [u8; 32],
    source_basis: ReleasedGenerationReclaimBasisV1,
}

impl TerminalHeadIdentityNonReissue<'_> {
    pub(in crate::physical_runtime) const fn key(&self) -> ReleaseCustodyHeadKeyV1 {
        self.key
    }

    pub(in crate::physical_runtime) const fn root(&self) -> RootPublicationCell {
        self.root
    }

    pub(in crate::physical_runtime) const fn declaration_record(&self) -> PersistedRecordIdentity {
        self.declaration_record
    }

    pub(in crate::physical_runtime) const fn declaration_frame_sha256(&self) -> [u8; 32] {
        self.declaration_frame_sha256
    }

    /// The released generation whose object identity the declaration claims.
    pub(in crate::physical_runtime) const fn source_basis(
        &self,
    ) -> ReleasedGenerationReclaimBasisV1 {
        self.source_basis
    }

    /// The root owner promotes this claim when it installs the retirement
    /// fence. The claim never leaves the fact.
    pub(in crate::physical_runtime) fn session_claim_mut(
        &mut self,
    ) -> &mut PhysicalBlobSessionClaim {
        &mut self.session_claim
    }
}

#[cfg(test)]
impl<'runtime> TerminalHeadIdentityNonReissue<'runtime> {
    pub(in crate::physical_runtime) fn fixture(
        declarations: MutexGuard<'runtime, ()>,
        key: ReleaseCustodyHeadKeyV1,
        root: RootPublicationCell,
        source_basis: ReleasedGenerationReclaimBasisV1,
    ) -> Self {
        Self {
            _declarations: declarations,
            session_claim: PhysicalBlobSessionClaim::fixture(source_basis.session()),
            key,
            root,
            declaration_record: PersistedRecordIdentity::new([1; 16], 1).expect("fixture record"),
            declaration_frame_sha256: [9; 32],
            source_basis,
        }
    }
}

/// The fold over every selected control record. Exactly one SessionDeclared
/// may name the basis, and it must name both its object and its session. No
/// GenerationPublished may name either.
struct IdentitySearch {
    basis: ReleasedGenerationReclaimBasisV1,
    selected: Option<BoundDeclaration>,
    conflict: bool,
    published: bool,
}

struct BoundDeclaration {
    record: PersistedRecordIdentity,
    frame_sha256: [u8; 32],
    maximum_checkpoint_sequence: u64,
}

impl IdentitySearch {
    const fn new(basis: ReleasedGenerationReclaimBasisV1) -> Self {
        Self {
            basis,
            selected: None,
            conflict: false,
            published: false,
        }
    }

    /// One selected control payload of the store named `store`.
    fn observe(
        &mut self,
        record: PersistedRecordIdentity,
        payload: &[u8],
        store: [u8; 16],
    ) -> Result<(), BlobReadOpenFailure> {
        if let Some(declaration) = selected_session_declaration(payload)? {
            if declaration.store() != store {
                return Err(BlobReadOpenFailure::ForeignStore);
            }
            self.observe_declaration(record, declaration, payload);
        } else if let Some(publication) = selected_generation_publication(payload)? {
            if publication.store() != store {
                return Err(BlobReadOpenFailure::ForeignStore);
            }
            self.published |= publication.object() == self.basis.object()
                || publication.session() == self.basis.session();
        }
        Ok(())
    }

    fn observe_declaration(
        &mut self,
        record: PersistedRecordIdentity,
        declaration: BlobSessionDeclarationV1,
        payload: &[u8],
    ) {
        let object = declaration.object() == self.basis.object();
        let session = declaration.session() == self.basis.session();
        if object || session {
            self.conflict |= self.selected.is_some() || !(object && session);
            self.selected = Some(BoundDeclaration {
                record,
                frame_sha256: Sha256::digest(payload).into(),
                maximum_checkpoint_sequence: declaration.max_checkpoint_sequence(),
            });
        }
    }

    /// `selected_checkpoint_sequence` is the completed durable checkpoint of
    /// the scanned root. Resume admits at or below the maximum, so only a
    /// sequence strictly above it proves the session can never resume.
    fn finish(
        self,
        selected_checkpoint_sequence: u64,
    ) -> Result<BoundDeclaration, TerminalHeadNonReissueDenial> {
        use TerminalHeadNonReissueDenial as Denial;
        if self.conflict {
            return Err(Denial::DeclarationConflict);
        }
        let bound = self.selected.ok_or(Denial::DeclarationAbsent)?;
        if self.published {
            return Err(Denial::PublicationSelected);
        }
        if selected_checkpoint_sequence <= bound.maximum_checkpoint_sequence {
            return Err(Denial::DeclarationNotExpired {
                selected_checkpoint_sequence,
                maximum_checkpoint_sequence: bound.maximum_checkpoint_sequence,
            });
        }
        Ok(bound)
    }
}

impl ServingPhysicalRuntime {
    /// Under the declaration lock, in the order begin uses: take the
    /// session's one runtime claim with its root and completed checkpoint,
    /// then make one complete pass over that claimed reader. Another holder
    /// of the session identity denies the claim. A bounded scan exhaustion is
    /// a denial, never absence or presence.
    pub(in crate::physical_runtime) fn attest_terminal_head_identity_non_reissue(
        &self,
        basis: ReleasedGenerationReclaimBasisV1,
        max_selected_records: NonZeroU64,
        scratch: &mut [u8],
    ) -> Result<
        (PhysicalRecordReader, TerminalHeadIdentityNonReissue<'_>),
        TerminalHeadNonReissueDenial,
    > {
        use TerminalHeadNonReissueDenial as Denial;
        let declarations = self.lock_blob_declaration();
        let key = ReleaseCustodyHeadKeyV1::new(basis.object(), basis.generation())
            .ok_or(Denial::InvalidKey)?;
        let (reader, session_claim, selected_checkpoint_sequence) = self
            .claimed_blob_reader(BlobSessionId::from_selected(basis.session()))
            .map_err(Denial::Claim)?;
        let root = reader.protected_root();
        let mut search = IdentitySearch::new(basis);
        let scan = walk_selected_reader(
            reader,
            max_selected_records,
            scratch,
            |record, payload, store| search.observe(record, payload, store.bytes()),
        )
        .map_err(Denial::Inspection)?;
        let bound = search.finish(selected_checkpoint_sequence)?;
        Ok((
            scan.into_protected_reader(),
            TerminalHeadIdentityNonReissue {
                _declarations: declarations,
                session_claim,
                key,
                root: root.root(),
                declaration_record: bound.record,
                declaration_frame_sha256: bound.frame_sha256,
                source_basis: basis,
            },
        ))
    }
}

#[cfg(test)]
#[path = "terminal_head_non_reissue/tests.rs"]
mod tests;
