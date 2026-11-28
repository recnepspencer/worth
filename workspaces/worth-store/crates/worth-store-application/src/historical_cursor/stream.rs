//! Shared canonical bounded chunk-by-chunk page stream and Home-MAC validation
//! used by both Directory and Memory retained-basis historical cursors
//! (Store Milestone 4, Phase M4.4).

use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{
    AdmittedHistoricalProjection, CanonicalCursorKey, HistoricalBasisRow, HistoricalQueryContract,
    HistoricalReadDenial, RetainedBasisTokenFields, SchemaVersionIdentity,
    RETAINED_BASIS_TOKEN_VERSION,
};

use super::authority::HistoricalCursorAuthority;
use super::aux_index::AuxiliaryIndexRegistry;
use super::basis_manifest::load_manifest_leaf_after;
use super::catalog::RetainedBasisRecord;
use super::chunk_format::{ChunkMemberId, SealedChunkDescriptor};
use super::chunk_scan::scan_verified_chunk_bounded;
use super::continuation_manifest::lookup_continuation_locator;
use super::residency::HistoricalWorkingSetPool;
use super::token::RetainedBasisToken;

/// Certification-only mutant selector driving real production `M7` mutant-kill courts.
#[cfg(feature = "certification-test-authority")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoricalCursorCertificationMutant {
    None,
    RebindToHeadOnPruned,
    SkipTokenMacVerification,
    ResumeByPhysicalOffset,
    SkipPinLoanAcquisition,
    LocatorCoupledCommitment,
    SkipQueryDigestValidation,
}

fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}

/// Shared precondition validation & Home-MAC token minting for `open_historical_cursor`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_open_token(
    authority: &HistoricalCursorAuthority,
    execution: &ExecutionRequest<'_, '_>,
    home_identity: HomeIdentity,
    home_incarnation: HomeIncarnation,
    commit_id: CommitId,
    root_semantic_commitment: [u8; 32],
    entry: &RetainedBasisRecord,
    query: &HistoricalQueryContract,
    supported_schema: &SchemaVersionIdentity,
    require_canonical_stream: bool,
) -> Result<RetainedBasisToken, HistoricalReadDenial> {
    if execution.consult().is_err() {
        return Err(HistoricalReadDenial::RequestStopped);
    }
    let proj_ok = if require_canonical_stream {
        query.admitted_canonical_projection().is_some()
    } else {
        query.admitted_projection().is_some()
    };
    if query.schema_version() != supported_schema || !proj_ok {
        return Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
            schema_version: query.schema_version().clone(),
        });
    }
    if entry.schema_version != *supported_schema {
        return Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
            schema_version: entry.schema_version.clone(),
        });
    }
    if entry.root_semantic_commitment != root_semantic_commitment {
        return Err(corrupt());
    }
    authority
        .issue_token(RetainedBasisTokenFields {
            home_identity,
            home_incarnation,
            branch_origin: entry.branch_origin.clone(),
            commit_id,
            root_semantic_commitment,
            schema_version: entry.schema_version.clone(),
            query_digest: query.query_digest(),
            canonical_key_position: CanonicalCursorKey::start(),
        })
        .ok_or_else(corrupt)
}

/// Shared pre-fence validation for `resume_cursor` across Directory and Memory stores.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_resume_preconditions(
    authority: &HistoricalCursorAuthority,
    execution: &ExecutionRequest<'_, '_>,
    home_identity: HomeIdentity,
    home_incarnation: HomeIncarnation,
    token: &RetainedBasisToken,
    expected_query: &HistoricalQueryContract,
    supported_schema: &SchemaVersionIdentity,
    require_canonical_stream: bool,
    verify_mac: bool,
    verify_query_digest: bool,
) -> Result<(), HistoricalReadDenial> {
    if execution.consult().is_err() {
        return Err(HistoricalReadDenial::RequestStopped);
    }
    if (verify_mac && !authority.verify_token(token))
        || token.format_version() != RETAINED_BASIS_TOKEN_VERSION
        || token.home_identity() != home_identity
        || token.home_incarnation() > home_incarnation
    {
        return Err(corrupt());
    }
    let query_ok = if require_canonical_stream {
        expected_query.admitted_canonical_projection().is_some()
    } else {
        expected_query.admitted_projection().is_some()
    };
    let digest_mismatch =
        verify_query_digest && *token.query_digest() != expected_query.query_digest();
    if !query_ok
        || token.schema_version() != supported_schema
        || expected_query.schema_version() != supported_schema
        || digest_mismatch
    {
        return Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
            schema_version: token.schema_version().clone(),
        });
    }
    Ok(())
}

/// Shared catalog-entry validation for `resume_cursor` under the store fence.
pub(crate) fn validate_resumed_entry(
    entry: &RetainedBasisRecord,
    token: &RetainedBasisToken,
    supported_schema: &SchemaVersionIdentity,
) -> Result<(), HistoricalReadDenial> {
    if entry.branch_origin != *token.branch_origin()
        || entry.root_semantic_commitment != *token.root_semantic_commitment()
    {
        return Err(corrupt());
    }
    if entry.schema_version != *supported_schema {
        return Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
            schema_version: entry.schema_version.clone(),
        });
    }
    Ok(())
}

/// Backend-supplied raw member loader bound to one active `ReaderLoan`.
pub(crate) trait LoanedMemberReader: Send {
    fn read_member(
        &self,
        id: ChunkMemberId,
        locator_bytes: Option<&[u8; 64]>,
    ) -> Result<Vec<u8>, HistoricalReadDenial>;
}

/// Streams canonical rows strictly after `after` in total [`CanonicalCursorKey`]
/// order, enforcing BOTH row `limit` and `streaming_window_bytes`, applying the
/// admitted projection, and charging every faulted chunk/continuation page to
/// the isolated [`HistoricalWorkingSetPool`].
pub(crate) struct CanonicalChunkCursorStream<'a> {
    commit_id: CommitId,
    root_semantic_commitment: [u8; 32],
    chunks: Option<Arc<[SealedChunkDescriptor]>>,
    manifest_ref: Option<(ChunkMemberId, Option<[u8; 64]>)>,
    query: HistoricalQueryContract,
    projection: AdmittedHistoricalProjection,
    residency: HistoricalWorkingSetPool,
    aux_indexes: Arc<AuxiliaryIndexRegistry>,
    reader: Box<dyn LoanedMemberReader + 'a>,
    #[cfg(feature = "certification-test-authority")]
    mutant: HistoricalCursorCertificationMutant,
}

impl<'a> CanonicalChunkCursorStream<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        commit_id: CommitId,
        root_semantic_commitment: [u8; 32],
        chunks: Arc<[SealedChunkDescriptor]>,
        query: HistoricalQueryContract,
        projection: AdmittedHistoricalProjection,
        residency: HistoricalWorkingSetPool,
        aux_indexes: Arc<AuxiliaryIndexRegistry>,
        reader: Box<dyn LoanedMemberReader + 'a>,
        #[cfg(feature = "certification-test-authority")]
        mutant: HistoricalCursorCertificationMutant,
    ) -> Self {
        Self {
            commit_id,
            root_semantic_commitment,
            chunks: Some(chunks),
            manifest_ref: None,
            query,
            projection,
            residency,
            aux_indexes,
            reader,
            #[cfg(feature = "certification-test-authority")]
            mutant,
        }
    }

    pub(crate) fn with_manifest_ref(
        mut self,
        manifest_ref: Option<(ChunkMemberId, Option<[u8; 64]>)>,
    ) -> Self {
        if manifest_ref.is_some() {
            self.chunks = None;
            self.manifest_ref = manifest_ref;
        }
        self
    }

    fn start_chunk_index(
        &self,
        chunks: &[SealedChunkDescriptor],
        after: &CanonicalCursorKey,
    ) -> usize {
        if after.is_start() {
            return 0;
        }
        self.aux_indexes
            .first_chunk_after(self.commit_id, self.root_semantic_commitment, after)
            .map_or_else(
                || chunks.partition_point(|chunk| &chunk.max_key <= after),
                |idx| idx.min(chunks.len()),
            )
    }

    fn load_continuation_member(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        chunk: &SealedChunkDescriptor,
        cont_id: ChunkMemberId,
        cont_loc: Option<&[u8; 64]>,
    ) -> Result<Arc<Vec<u8>>, HistoricalReadDenial> {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        let resolved_loc = if let Some(cont_root) = chunk.continuation_manifest_member {
            lookup_continuation_locator(
                cont_root,
                chunk.continuation_manifest_locator.as_ref(),
                cont_id,
                |id, loc| {
                    if execution.consult().is_err() {
                        return Err(HistoricalReadDenial::RequestStopped);
                    }
                    self.residency
                        .load_historical_manifest(id, || self.reader.read_member(id, loc))
                },
            )?
            .map(Some)
            .ok_or_else(corrupt)?
        } else {
            cont_loc.copied()
        };
        self.residency.load_historical_member(cont_id, true, || {
            self.reader.read_member(cont_id, resolved_loc.as_ref())
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn scan_chunk_slice(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        chunks: &[SealedChunkDescriptor],
        effective_after: &CanonicalCursorKey,
        limit: usize,
        streaming_window: usize,
        accumulated_bytes: &mut usize,
        collected: &mut Vec<(CanonicalCursorKey, HistoricalBasisRow)>,
    ) -> Result<bool, HistoricalReadDenial> {
        for chunk in chunks {
            if execution.consult().is_err() {
                return Err(HistoricalReadDenial::RequestStopped);
            }
            if !effective_after.is_start() && &chunk.max_key <= effective_after {
                continue;
            }
            let chunk_bytes =
                self.residency
                    .load_historical_member(chunk.chunk_member, false, || {
                        self.reader
                            .read_member(chunk.chunk_member, chunk.chunk_locator.as_ref())
                    })?;
            let chunk_has_more = scan_verified_chunk_bounded(
                chunk,
                &chunk_bytes,
                effective_after,
                &self.query,
                self.projection,
                limit,
                streaming_window,
                accumulated_bytes,
                collected,
                |id, loc| self.load_continuation_member(execution, chunk, id, loc),
            )?;
            drop(chunk_bytes);
            if chunk_has_more {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn read_page(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        after: &CanonicalCursorKey,
        limit: usize,
    ) -> Result<(Vec<(CanonicalCursorKey, HistoricalBasisRow)>, bool), HistoricalReadDenial> {
        if execution.consult().is_err() {
            return Err(HistoricalReadDenial::RequestStopped);
        }
        #[cfg(feature = "certification-test-authority")]
        let offset_mutant = self.mutant
            == HistoricalCursorCertificationMutant::ResumeByPhysicalOffset
            && !after.is_start();
        #[cfg(not(feature = "certification-test-authority"))]
        let offset_mutant = false;

        let win = (self.residency.budget().streaming_window_bytes as usize).max(1);
        let (mut collected, mut acc) = (Vec::with_capacity(limit.min(64)), 0usize);
        if let Some(chunks) = self.chunks.as_ref() {
            let mut start = self.start_chunk_index(chunks, after);
            let mut eff = after.clone();
            if offset_mutant && start + 1 < chunks.len() {
                start += 1;
                eff = CanonicalCursorKey::start();
            }
            let more = self.scan_chunk_slice(
                execution,
                &chunks[start..],
                &eff,
                limit,
                win,
                &mut acc,
                &mut collected,
            )?;
            return Ok((collected, more));
        }

        let Some((root_id, root_loc)) = self.manifest_ref else {
            return Ok((collected, false));
        };
        let (mut seek, mut eff, mut mutated) = (after.clone(), after.clone(), false);
        while let Some(leaf) =
            load_manifest_leaf_after(root_id, root_loc.as_ref(), &seek, |id, loc| {
                if execution.consult().is_err() {
                    return Err(HistoricalReadDenial::RequestStopped);
                }
                self.residency
                    .load_historical_manifest(id, || self.reader.read_member(id, loc))
            })?
        {
            let mut start = if seek.is_start() {
                0
            } else {
                leaf.partition_point(|chunk| &chunk.max_key <= &seek)
            };
            if offset_mutant && !mutated {
                mutated = true;
                eff = CanonicalCursorKey::start();
                if start + 1 < leaf.len() {
                    start += 1;
                } else if let Some(last) = leaf.last() {
                    seek = last.max_key.clone();
                    continue;
                }
            }
            if self.scan_chunk_slice(
                execution,
                &leaf[start..],
                &eff,
                limit,
                win,
                &mut acc,
                &mut collected,
            )? {
                return Ok((collected, true));
            }
            let Some(last) = leaf.last() else { break };
            seek = last.max_key.clone();
        }
        Ok((collected, false))
    }
}
