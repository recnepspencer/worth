//! Emit only independently authenticated groups, funded by the invoking request.
use super::*;

pub(in super::super) fn encode(
    cut: &CheckpointAdmissionCut,
    roots: &ExecutionImmutableBytes,
    rows: &[EncodedAdmissionRow],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(ExecutionImmutableBytes, DecodedDirectory), Refusal> {
    if rows.len() as u64 != cut.canonical_through() {
        return Err(Refusal::InvalidProjectedMeaning);
    }
    let description = cut.clone();
    let cut = crate::durability::artifact_payload_emission::request::encode_named_charged(
        &[],
        &super::super::binary_encoding::BinaryFixedArrays(cut),
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
    let mut output =
        ExecutionRequestArrayBuilder::allocate(*request, size).map_err(Refusal::Buffer)?;
    append(&mut output, &header, request)?;
    append(&mut output, cut.bytes(), request)?;
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
    append(&mut output, &root_header, request)?;
    let root_start = output.len();
    append(&mut output, roots.bytes(), request)?;
    let root_range = root_start..output.len();
    let mut indexes =
        ExecutionRequestArrayBuilder::allocate(*request, rows.len()).map_err(Refusal::Buffer)?;
    let mut commits = ExecutionOrderedMap::default();
    let mut previous = 0;
    for (ordinal, row) in rows.iter().enumerate() {
        if row.key.position == 0
            || row.key.commit == 0
            || row.key.position <= previous
            || row.key.position > description.canonical_through()
            || commits.contains_key(&row.key.commit)
        {
            return Err(Refusal::InvalidProjectedMeaning);
        }
        commits
            .insert_cloned_native_for_request(&row.key.commit, &ordinal, *request)
            .map_err(|cause| {
                super::super::refusal::native_stop(
                    crate::durability::checkpoint_history::capture::map_stop(cause),
                )
            })?;
        authentication::update(&mut directory, &row.key.position.to_le_bytes(), request)?;
        authentication::update(&mut directory, &row.key.commit.to_le_bytes(), request)?;
        append(&mut output, &row.key.position.to_le_bytes(), request)?;
        append(&mut output, &row.key.commit.to_le_bytes(), request)?;
        let mut groups = std::array::from_fn(|_| 0..0);
        for (index, (meaning, payload)) in
            CANONICAL_MEANINGS.into_iter().zip(&row.groups).enumerate()
        {
            let group = group_header(cut.bytes(), row.key, meaning, payload.bytes(), request)?;
            authentication::update(&mut directory, &group, request)?;
            append(&mut output, &group, request)?;
            let start = output.len();
            append(&mut output, payload.bytes(), request)?;
            groups[index] = start..output.len();
        }
        indexes
            .push(AdmissionRowIndex {
                key: row.key,
                groups,
            })
            .map_err(Refusal::Buffer)?;
        previous = row.key.position;
    }
    let digest = directory.finalize();
    account(request, digest.len() as u64)?;
    for (index, byte) in digest.iter().copied().enumerate() {
        output
            .replace_initialized(32 + index, byte)
            .map_err(Refusal::Buffer)?;
    }
    let positions = super::super::root_positions::read(roots.bytes(), &commits, request)?;
    let indexes = indexes.seal().map_err(Refusal::Buffer)?;
    let bytes = output.seal_immutable_bytes().map_err(Refusal::Buffer)?;
    Ok((
        bytes,
        DecodedDirectory {
            cut: description,
            roots: root_range,
            rows: indexes,
            commits,
            positions,
        },
    ))
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
