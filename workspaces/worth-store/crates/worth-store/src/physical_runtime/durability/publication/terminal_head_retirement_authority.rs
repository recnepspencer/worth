//! The join of the four owner-issued exclusion facts with the checkpoint
//! attestation for one terminal head retired. Each input is sealed in its
//! owner's module, so a terminal flag, a missing route or checkpoint age
//! alone cannot build this authority.

use worth_store_physical_format::{
    PersistedRecordIdentity, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    ReleasedGenerationReclaimBasisV1, RootPublicationCell,
};

use super::current_root_owner::{
    AdmittedTerminalHeadRetirement, PhysicalReclaimAttempt, TerminalHeadPublicationExcluded,
};
use crate::physical_runtime::{
    blob::TerminalHeadIdentityNonReissue, durability::TerminalHeadNoRetryClaim,
    stability::TerminalHeadNoReaderOrRecoveryHold,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum TerminalHeadRetirementAuthorityDenial {
    KeyMismatch,
    RootMismatch,
}

/// Exists only while the retirement fence, the declaration lock and the
/// session's runtime claim are held. The root owner's facts stay bound to
/// their fence attempt inside the admitted value.
pub(in crate::physical_runtime) struct TerminalHeadRetirementAuthority<'runtime> {
    admitted: AdmittedTerminalHeadRetirement,
    identity: TerminalHeadIdentityNonReissue<'runtime>,
}

impl<'runtime> TerminalHeadRetirementAuthority<'runtime> {
    pub(in crate::physical_runtime) fn new(
        admitted: AdmittedTerminalHeadRetirement,
        retry: TerminalHeadNoRetryClaim,
        identity: TerminalHeadIdentityNonReissue<'runtime>,
    ) -> Result<Self, TerminalHeadRetirementAuthorityDenial> {
        let head = admitted.head();
        same_subject(
            head.key(),
            head.root(),
            admitted.publication(),
            admitted.hold(),
            &retry,
            &identity,
        )?;
        Ok(Self { admitted, identity })
    }

    pub(in crate::physical_runtime) const fn attempt(&self) -> &PhysicalReclaimAttempt {
        self.admitted.attempt()
    }

    pub(in crate::physical_runtime) fn key(&self) -> ReleaseCustodyHeadKeyV1 {
        self.admitted.head().key()
    }

    pub(in crate::physical_runtime) const fn root(&self) -> RootPublicationCell {
        self.admitted.head().root()
    }

    pub(in crate::physical_runtime) const fn expected_prior(&self) -> ReleaseCustodyHeadEntryV1 {
        self.admitted.head().entry()
    }

    pub(in crate::physical_runtime) const fn declaration_record(&self) -> PersistedRecordIdentity {
        self.identity.declaration_record()
    }

    pub(in crate::physical_runtime) const fn declaration_frame_sha256(&self) -> [u8; 32] {
        self.identity.declaration_frame_sha256()
    }

    pub(in crate::physical_runtime) const fn source_basis(
        &self,
    ) -> ReleasedGenerationReclaimBasisV1 {
        self.identity.source_basis()
    }
}

/// Every fact must name the attested head's key and the attested root.
fn same_subject(
    key: ReleaseCustodyHeadKeyV1,
    root: RootPublicationCell,
    publication: &TerminalHeadPublicationExcluded,
    hold: &TerminalHeadNoReaderOrRecoveryHold,
    retry: &TerminalHeadNoRetryClaim,
    identity: &TerminalHeadIdentityNonReissue<'_>,
) -> Result<(), TerminalHeadRetirementAuthorityDenial> {
    if [publication.key(), hold.key(), retry.key(), identity.key()]
        .iter()
        .any(|named| *named != key)
    {
        return Err(TerminalHeadRetirementAuthorityDenial::KeyMismatch);
    }
    if [
        publication.root(),
        hold.root(),
        retry.root(),
        identity.root(),
    ]
    .iter()
    .any(|named| *named != root)
    {
        return Err(TerminalHeadRetirementAuthorityDenial::RootMismatch);
    }
    Ok(())
}

#[cfg(test)]
#[path = "terminal_head_retirement_authority/tests.rs"]
mod tests;
