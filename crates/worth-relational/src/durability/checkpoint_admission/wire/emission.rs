//! Emit only independently authenticated groups, funded by the invoking request.
use super::*;

pub(in super::super) fn encode(
    cut: &CheckpointAdmissionCut,
    roots: &ExecutionImmutableBytes,
    rows: &[EncodedAdmissionRow],
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, Refusal> {
    let cut = crate::durability::artifact_payload_emission::request::encode_named_charged(
        &[],
        cut,
        request,
    )
    .map_err(super::super::refusal::native_stop)?;
    if cut.len() > MAXIMUM_CUT {
        return Err(Refusal::CapacityOverflow);
    }
    let mut size = HEADER
        .checked_add(cut.len())
        .and_then(|size| size.checked_add(GROUP_HEADER))
        .and_then(|size| size.checked_add(roots.len()))
        .ok_or(Refusal::CapacityOverflow)?;
    for row in rows {
        account(request, 1)?;
        size = size
            .checked_add(MINIMUM_ROW)
            .ok_or(Refusal::CapacityOverflow)?;
        for group in &row.groups {
            account(request, 1)?;
            size = size
                .checked_add(group.len())
                .ok_or(Refusal::CapacityOverflow)?;
        }
    }
    if size > 1 << 30 {
        return Err(Refusal::CapacityOverflow);
    }
    let mut header = [0_u8; HEADER];
    header[..8].copy_from_slice(MAGIC);
    header[8..12].copy_from_slice(&VERSION.get().to_le_bytes());
    header[12..16].copy_from_slice(&(cut.len() as u32).to_le_bytes());
    header[16..24].copy_from_slice(&(rows.len() as u64).to_le_bytes());
    header[24..32].copy_from_slice(&(roots.len() as u64).to_le_bytes());
    let mut directory = authentication::directory(&header[..32], cut.bytes(), request)?;
    let root_key = AdmissionRowKey {
        position: 0,
        commit: 0,
    };
    let root_header = group_header(
        cut.bytes(),
        root_key,
        Meaning::RootDirectory,
        roots.bytes(),
        request,
    )?;
    authentication::update(&mut directory, &root_header, request)?;
    for row in rows {
        authentication::update(&mut directory, &row.key.position.to_le_bytes(), request)?;
        authentication::update(&mut directory, &row.key.commit.to_le_bytes(), request)?;
        for (meaning, payload) in CANONICAL_MEANINGS.into_iter().zip(&row.groups) {
            let group = group_header(cut.bytes(), row.key, meaning, payload.bytes(), request)?;
            authentication::update(&mut directory, &group, request)?;
        }
    }
    header[32..].copy_from_slice(&directory.finalize());
    let mut output =
        ExecutionRequestArrayBuilder::allocate(*request, size).map_err(Refusal::Buffer)?;
    append(&mut output, &header, request)?;
    append(&mut output, cut.bytes(), request)?;
    append(&mut output, &root_header, request)?;
    append(&mut output, roots.bytes(), request)?;
    for row in rows {
        append(&mut output, &row.key.position.to_le_bytes(), request)?;
        append(&mut output, &row.key.commit.to_le_bytes(), request)?;
        for (meaning, payload) in CANONICAL_MEANINGS.into_iter().zip(&row.groups) {
            let group = group_header(cut.bytes(), row.key, meaning, payload.bytes(), request)?;
            append(&mut output, &group, request)?;
            append(&mut output, payload.bytes(), request)?;
        }
    }
    output.seal_immutable_bytes().map_err(Refusal::Buffer)
}

fn group_header(
    cut: &[u8],
    key: AdmissionRowKey,
    meaning: Meaning,
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; GROUP_HEADER], Refusal> {
    let mut header = [0_u8; GROUP_HEADER];
    header[0] = meaning as u8;
    header[1..9].copy_from_slice(&(payload.len() as u64).to_le_bytes());
    header[9..].copy_from_slice(&authentication::meaning(
        cut, key, meaning, payload, request,
    )?);
    Ok(header)
}

fn append(
    output: &mut ExecutionRequestArrayBuilder<'_, '_, u8>,
    bytes: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    for chunk in bytes.chunks(64 * 1024) {
        account(request, chunk.len() as u64)?;
        output.extend_from_slice(chunk).map_err(Refusal::Buffer)?;
    }
    Ok(())
}

fn account(request: &ExecutionRequest<'_, '_>, work: u64) -> Result<(), Refusal> {
    crate::durability::artifact_payload_emission::request::account_work(request, work)
        .map_err(super::super::refusal::native_stop)
}
