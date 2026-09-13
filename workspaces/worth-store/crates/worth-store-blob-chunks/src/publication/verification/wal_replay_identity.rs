use worth_store_wal::{PublicationScope, WalFramePublicationScope};

use super::super::types::reachability_staging::BlobReachabilityStagingIdentity;
use super::super::types::{BlobPublicationWalCommit, BlobPublicationWalPayload};
use super::super::{
    BlobPublicationCounterSnapshot, BlobPublicationDenial, BlobPublicationDurableWal,
    BlobPublicationIntent,
};

pub(crate) fn verify_staging_payload_match(
    payload: &BlobPublicationWalPayload,
    staging_identity: &BlobReachabilityStagingIdentity,
    counters: BlobPublicationCounterSnapshot,
) -> Result<(), BlobPublicationDenial> {
    if payload.staging_identity() == staging_identity {
        Ok(())
    } else {
        Err(BlobPublicationDenial::WalReplayIdentityMismatch { counters })
    }
}

pub(crate) fn verify_wal_frame_scope(
    publication_declaration: &worth_store_wal::PublicationDeclaration,
    counters: BlobPublicationCounterSnapshot,
) -> Result<WalFramePublicationScope, BlobPublicationDenial> {
    let PublicationScope::WalFrame(wal_scope) = publication_declaration.scope() else {
        return Err(BlobPublicationDenial::WalPublicationScopeRequired { counters });
    };
    Ok(wal_scope.clone())
}

pub(crate) fn require_matching_replay_identity(
    declared: &WalFramePublicationScope,
    replayed: &BlobPublicationDurableWal,
    payload: &BlobPublicationWalPayload,
    counters: BlobPublicationCounterSnapshot,
) -> Result<(), BlobPublicationDenial> {
    let replayed_range = replayed.lsn_range();
    let identities_match = declared.segment_id() == replayed.segment_id().get()
        && declared.generation() == replayed.generation().get()
        && declared.lsn_start() == replayed_range.start().get()
        && declared.lsn_end() == replayed_range.end_exclusive().get()
        && declared.frame_digest() == replayed.frame_digest().as_str()
        && declared.frame_digest() == payload.frame_digest()
        && declared.expected_bytes() == replayed.expected_bytes();
    if identities_match {
        Ok(())
    } else {
        Err(BlobPublicationDenial::WalReplayIdentityMismatch { counters })
    }
}

// silence unused import warning for WalCommit in module scope during refactors
#[allow(dead_code)]
type _WalCommit = BlobPublicationWalCommit;
#[allow(dead_code)]
type _Intent = BlobPublicationIntent;
