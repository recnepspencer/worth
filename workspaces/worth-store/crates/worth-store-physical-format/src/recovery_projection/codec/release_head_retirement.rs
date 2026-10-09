//! Wire codec for a terminal head retirement. The body carries the
//! `RetireTerminal` mutation under its own tag and never an upsert.

use super::head_effect::{
    fund_exact_verification, read_entry, read_optional_ref, read_ref, read_tree_frames,
    write_entry, write_optional_ref, write_ref, write_tree_frames, RETIRE_TERMINAL_MUTATION_TAG,
};
use super::*;
use crate::{
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionV1, ReleasedGenerationReclaimBasisV1,
};

pub(super) fn encode_terminal_head_retirement(
    retirement: &PersistedTerminalReleaseHeadRetirementV1,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&retirement.tree_identity().to_le_bytes());
    field(&mut bytes, &retirement.source_basis().encode());
    write_ref(&mut bytes, retirement.source_root());
    bytes.extend_from_slice(&retirement.source_next_block().to_le_bytes());
    bytes.push(RETIRE_TERMINAL_MUTATION_TAG);
    write_entry(&mut bytes, retirement.expected_prior());
    write_optional_ref(&mut bytes, retirement.result_root());
    bytes.extend_from_slice(&retirement.result_next_block().to_le_bytes());
    write_tree_frames(
        &mut bytes,
        retirement.source_path(),
        retirement.node_writes(),
    );
    bytes
}

pub(super) fn decode_terminal_head_retirement<S: PhysicalRecoveryDecodeStorage>(
    bytes: &[u8],
    declaration_record: PersistedRecordIdentity,
    declaration_frame_sha256: [u8; 32],
    source_root_generation: u64,
    remaining_entries: &mut u64,
    format: crate::PhysicalRecordFormatDeclaration,
    storage: &mut S,
) -> Result<PersistedTerminalReleaseHeadRetirementV1, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let mut cursor = Cursor::new(bytes);
    let tree_identity = cursor.u64()?;
    let source_basis = ReleasedGenerationReclaimBasisV1::decode(cursor.field()?)
        .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
    let source_root = read_ref(&mut cursor)?;
    let source_next_block = cursor.u64()?;
    if cursor.byte()? != RETIRE_TERMINAL_MUTATION_TAG {
        return Err(PhysicalRecoveryProjectionDenial::Malformed.into());
    }
    let expected_prior = read_entry(&mut cursor)?;
    let result_root = read_optional_ref(&mut cursor)?;
    let result_next_block = cursor.u64()?;
    let path = read_tree_frames(
        &mut cursor,
        remaining_entries,
        format,
        storage,
        ReleaseCustodyHeadPathNodeV1::new,
    )?;
    let writes = read_tree_frames(
        &mut cursor,
        remaining_entries,
        format,
        storage,
        ReleaseCustodyHeadNodeWriteV1::new,
    )?;
    cursor.end()?;
    let result_generation = source_root_generation
        .checked_add(1)
        .ok_or(PhysicalRecoveryProjectionDenial::Malformed)?;
    let new_blocks = PersistedTerminalReleaseHeadRetirementV1::reserved_new_blocks(&writes);
    let limits = release_head_tree_claim::exact_limits(&path, new_blocks, format)?;
    fund_exact_verification(&path, new_blocks, format, storage)?;
    let planned = ReleaseCustodyHeadTransitionV1::verify_exact(
        Some(source_root),
        source_next_block,
        &path,
        ReleaseCustodyHeadMutationV1::RetireTerminal { expected_prior },
        result_generation,
        tree_identity,
        format,
        limits,
        result_root,
        result_next_block,
        &writes,
    )
    .map_err(|_| PhysicalRecoveryProjectionDenial::Malformed)?;
    PersistedTerminalReleaseHeadRetirementV1::new(
        source_root_generation,
        declaration_record,
        declaration_frame_sha256,
        tree_identity,
        source_basis,
        path,
        planned,
        format,
    )
    .map_err(Into::into)
}
