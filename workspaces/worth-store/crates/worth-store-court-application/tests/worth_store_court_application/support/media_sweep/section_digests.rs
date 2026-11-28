//! Cryptographic verification and deterministic hashing of the non-`WRTHRCS1`
//! continuation-relevant checkpoint sections inside `WAPHCKP1` v3:
//! - `WSRCAI01` child 1: `WRTHCAI1` v1 (Relational admission image)
//! - `WSRCAI01` child 2: `WRWMREG1` v2 (World admission companion)
//! - `WAPHCKP1` section 1: `WWRLDCP1` (World checkpoint section)
//! - `WAPHCKP1` section 2: `WSIGOWNR` v2 kind 5 (Signal checkpoint section)
//!
//! Any malformed framing, checksum mismatch, cross-section binding drift, or
//! non-empty World table fails closed (`None`) so World continuation runs.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::manifest_digest::{encode_canonical_home_cut, hash_normalized_relational_bytes, u64_le};
use super::signal_digest::{
    hash_normalized_json, hash_replacing_identities, read_slice, read_u64, verify_wsigownr_inner,
};

pub(crate) struct Wsrcai01Bindings<'a> {
    pub(crate) home: &'a [u8],
    pub(crate) covered: &'a [u8],
    pub(crate) covered_transitions: &'a [u8],
    pub(crate) native_origin: &'a [u8],
    pub(crate) world_through: &'a [u8],
    pub(crate) checkpoint_digest: &'a [u8],
    pub(crate) world_image_digest: &'a [u8],
    pub(crate) source_digest: &'a [u8],
    pub(crate) image_digest: &'a [u8],
}

pub(super) fn verify_and_hash_wrthcai1(
    image: &[u8],
    b: &Wsrcai01Bindings<'_>,
    out: &mut Sha256,
) -> Option<()> {
    const HEADER: usize = 64;
    if image.len() < HEADER || &image[..8] != b"WRTHCAI1" {
        return None;
    }
    if u32::from_le_bytes(image[8..12].try_into().ok()?) != 1 {
        return None;
    }
    let cut_len = u32::from_le_bytes(image[12..16].try_into().ok()?) as usize;
    let row_count = u64_le(image, 16)?;
    let root_len = u64_le(image, 24)?;
    let covered = u64::from_le_bytes(b.covered.try_into().ok()?);
    if row_count as u64 != covered {
        return None;
    }
    let cut_end = HEADER.checked_add(cut_len)?;
    let cut_bytes = image.get(HEADER..cut_end)?;
    if cut_bytes != encode_canonical_home_cut(b).as_slice() {
        return None;
    }

    let mut directory = Sha256::new();
    directory.update(b"worth.relational.checkpoint-admission.directory.v1\0");
    directory.update(&image[..32]);
    directory.update(cut_bytes);

    out.update(b"WRTHCAI1");
    out.update((row_count as u64).to_le_bytes());

    let mut offset = cut_end;
    let roots_payload = verify_group(image, &mut offset, cut_bytes, 0, 0, 9, &mut directory)?;
    if roots_payload.len() != root_len {
        return None;
    }
    hash_normalized_relational_bytes(roots_payload, out)?;
    out.update(b"\0/roots");

    for _ in 0..row_count {
        let pos_bytes = image.get(offset..offset + 8)?;
        let commit_bytes = image.get(offset + 8..offset + 16)?;
        let position = u64::from_le_bytes(pos_bytes.try_into().ok()?);
        let commit = u64::from_le_bytes(commit_bytes.try_into().ok()?);
        offset = offset.checked_add(16)?;
        directory.update(pos_bytes);
        directory.update(commit_bytes);
        out.update(pos_bytes);
        out.update(commit_bytes);
        for meaning in 1u8..=8 {
            let payload = verify_group(
                image,
                &mut offset,
                cut_bytes,
                position,
                commit,
                meaning,
                &mut directory,
            )?;
            out.update([meaning]);
            hash_normalized_relational_bytes(payload, out)?;
            out.update(b"\0/grp");
        }
    }
    if offset != image.len() || directory.finalize().as_slice() != &image[32..HEADER] {
        return None;
    }
    Some(())
}

fn verify_group<'a>(
    raw: &'a [u8],
    offset: &mut usize,
    cut: &[u8],
    position: u64,
    commit: u64,
    expected_meaning: u8,
    directory: &mut Sha256,
) -> Option<&'a [u8]> {
    let start = *offset;
    let header_end = start.checked_add(41)?;
    let header = raw.get(start..header_end)?;
    if header[0] != expected_meaning {
        return None;
    }
    let len = u64_le(header, 1)?;
    let end = header_end.checked_add(len)?;
    let payload = raw.get(header_end..end)?;

    let mut h = Sha256::new();
    h.update(b"worth.relational.checkpoint-admission.meaning.v1\0");
    h.update(1u32.to_le_bytes());
    h.update((cut.len() as u64).to_le_bytes());
    h.update(cut);
    h.update(position.to_le_bytes());
    h.update(commit.to_le_bytes());
    h.update([expected_meaning]);
    h.update((payload.len() as u64).to_le_bytes());
    h.update(payload);
    if h.finalize().as_slice() != &header[9..41] {
        return None;
    }
    directory.update(header);
    *offset = end;
    Some(payload)
}

pub(super) fn verify_and_hash_world(
    world: &[u8],
    b: &Wsrcai01Bindings<'_>,
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) -> Option<[BTreeMap<u64, [u8; 32]>; 2]> {
    const HEADER: usize = 52;
    if world.len() < HEADER || &world[..8] != b"WWRLDCP1" {
        return None;
    }
    if Sha256::digest(world).as_slice() != b.world_image_digest {
        return None;
    }
    let payload_len = u64_le(world, 12)?;
    if HEADER.checked_add(payload_len)? != world.len() {
        return None;
    }
    let p = &world[HEADER..];
    if Sha256::digest(p).as_slice() != &world[20..HEADER] {
        return None;
    }
    if p.len() < 229 || &p[4..20] != b.home {
        return None;
    }
    let mut cov_pos = [0u64; 3];
    let mut cov_digest = [[0u8; 32]; 3];
    for i in 0..3 {
        let pos_off = 68 + i * 40;
        if &p[pos_off..pos_off + 8] != &b.world_through[i * 8..(i + 1) * 8] {
            return None;
        }
        cov_pos[i] = u64::from_le_bytes(p[pos_off..pos_off + 8].try_into().ok()?);
        cov_digest[i].copy_from_slice(&p[pos_off + 8..pos_off + 40]);
    }
    if cov_pos[2] != 0 || cov_digest[2] != [0u8; 32] {
        return None;
    }
    let mut branches_digests = BTreeMap::new();
    let mut custody_digests = BTreeMap::new();
    let mut cursor = 228usize;
    let owner_tag = *p.get(cursor)?;
    cursor += 1;
    let owner_pos: &[u8] = match owner_tag {
        0 => &[],
        1 => {
            let pos_bytes = p.get(cursor..cursor + 8)?;
            let pos = u64::from_le_bytes(pos_bytes.try_into().ok()?);
            let digest: [u8; 32] = p.get(cursor + 8..cursor + 40)?.try_into().ok()?;
            insert_world_loc(&mut branches_digests, pos, digest)?;
            cursor += 40;
            pos_bytes
        }
        _ => return None,
    };

    out.update(b"WWRLDCP1");
    out.update(&world[8..12]);
    out.update(&p[0..4]);
    out.update(&p[20..68]);
    out.update(&p[188..228]);
    out.update([owner_tag]);
    out.update(owner_pos);

    for table in 0..2 {
        let count = read_u64(p, &mut cursor)?;
        out.update((count as u64).to_le_bytes());
        let map = if table == 0 {
            &mut branches_digests
        } else {
            &mut custody_digests
        };
        for _ in 0..count {
            let pos_bytes = read_slice(p, &mut cursor, 8)?;
            let pos = u64::from_le_bytes(pos_bytes.try_into().ok()?);
            let digest: [u8; 32] = read_slice(p, &mut cursor, 32)?.try_into().ok()?;
            insert_world_loc(map, pos, digest)?;
            let row_len = read_u64(p, &mut cursor)?;
            let row = read_slice(p, &mut cursor, row_len)?;
            out.update(pos_bytes);
            hash_world_semantic_row(row, b.home, sig_digests, out)?;
        }
    }
    let disp_count = read_u64(p, &mut cursor)?;
    out.update((disp_count as u64).to_le_bytes());
    for _ in 0..disp_count {
        let pos_bytes = read_slice(p, &mut cursor, 8)?;
        let pos = u64::from_le_bytes(pos_bytes.try_into().ok()?);
        let digest: [u8; 32] = read_slice(p, &mut cursor, 32)?.try_into().ok()?;
        insert_world_loc(&mut custody_digests, pos, digest)?;
        let attempt = read_slice(p, &mut cursor, 8)?;
        let tag = *read_slice(p, &mut cursor, 1)?.first()?;
        out.update(pos_bytes);
        out.update(attempt);
        out.update([tag]);
        match tag {
            0 | 2 | 3 => {}
            1 => {
                let ev_len = read_u64(p, &mut cursor)?;
                let ev = read_slice(p, &mut cursor, ev_len)?;
                hash_replacing_identities(ev, b.home, sig_digests, out);
            }
            _ => return None,
        }
    }
    validate_world_coverage(&branches_digests, cov_pos[0], cov_digest[0])?;
    validate_world_coverage(&custody_digests, cov_pos[1], cov_digest[1])?;
    let occ_count = read_u64(p, &mut cursor)?;
    if occ_count != 0 {
        return None;
    }
    out.update(0u64.to_le_bytes());
    let history_count = read_u64(p, &mut cursor)?;
    out.update((history_count as u64).to_le_bytes());
    for _ in 0..history_count {
        out.update(read_slice(p, &mut cursor, 8)?);
        hash_world_image_head(p, &mut cursor, b.home, out)?;
    }
    let heads_count = read_u64(p, &mut cursor)?;
    out.update((heads_count as u64).to_le_bytes());
    for _ in 0..heads_count {
        hash_world_branch(p, &mut cursor, b.home, out)?;
        hash_world_image_head(p, &mut cursor, b.home, out)?;
    }
    (cursor == p.len()).then_some([branches_digests, custody_digests])
}

fn insert_world_loc(map: &mut BTreeMap<u64, [u8; 32]>, pos: u64, digest: [u8; 32]) -> Option<()> {
    if pos == 0 || map.contains_key(&pos) {
        return None;
    }
    map.insert(pos, digest);
    Some(())
}

fn validate_world_coverage(
    map: &BTreeMap<u64, [u8; 32]>,
    cov_pos: u64,
    cov_digest: [u8; 32],
) -> Option<()> {
    if map.len() as u64 != cov_pos {
        return None;
    }
    let mut expected_pos = 0u64;
    let mut last_digest = [0u8; 32];
    for (&pos, &digest) in map {
        expected_pos += 1;
        if pos != expected_pos {
            return None;
        }
        last_digest = digest;
    }
    (last_digest == cov_digest).then_some(())
}

pub(super) fn verify_world_journal_entry(
    entry: &[u8],
    home: &[u8],
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) -> Option<()> {
    const HEADER: usize = 52;
    if entry.len() < HEADER || &entry[..8] != b"WWRLDART" {
        return None;
    }
    let payload_len = u64_le(entry, 12)?;
    if HEADER.checked_add(payload_len)? != entry.len() {
        return None;
    }
    let payload = &entry[HEADER..];
    if Sha256::digest(payload).as_slice() != &entry[20..HEADER] {
        return None;
    }
    out.update(&entry[8..12]);
    hash_world_semantic_row(payload, home, sig_digests, out)
}

fn hash_world_semantic_row(
    row: &[u8],
    home: &[u8],
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) -> Option<()> {
    let tag = *row.first()?;
    out.update([tag]);
    if tag != 2 {
        hash_replacing_identities(row, home, sig_digests, out);
        return Some(());
    }
    // WorldRecord::Branch: tag(1) + head + bootstrap(8) + program_revision(32)
    // + initial_components(relational_json, signal_wsigownr, bridge_json).
    let mut c = 1usize;
    hash_world_image_head(row, &mut c, home, out)?;
    out.update(read_slice(row, &mut c, 40)?);
    let rel_len = read_u64(row, &mut c)?;
    let rel_json = read_slice(row, &mut c, rel_len)?;
    hash_normalized_json(rel_json, home, sig_digests, out);
    let sig_len = read_u64(row, &mut c)?;
    let sig_frame = read_slice(row, &mut c, sig_len)?;
    verify_wsigownr_inner(sig_frame, home, sig_digests, out)?;
    let bridge_len = read_u64(row, &mut c)?;
    let bridge_json = read_slice(row, &mut c, bridge_len)?;
    hash_normalized_json(bridge_json, home, sig_digests, out);
    (c == row.len()).then_some(())
}

fn hash_world_branch(
    bytes: &[u8],
    cursor: &mut usize,
    home: &[u8],
    out: &mut Sha256,
) -> Option<()> {
    if read_slice(bytes, cursor, 16)? != home {
        return None;
    }
    let slot = read_slice(bytes, cursor, 8)?;
    let name_len = usize::from(u16::from_le_bytes(
        read_slice(bytes, cursor, 2)?.try_into().ok()?,
    ));
    let name = read_slice(bytes, cursor, name_len)?;
    out.update(slot);
    out.update((name_len as u16).to_le_bytes());
    out.update(name);
    Some(())
}

fn hash_world_image_head(
    bytes: &[u8],
    cursor: &mut usize,
    home: &[u8],
    out: &mut Sha256,
) -> Option<()> {
    hash_world_branch(bytes, cursor, home, out)?;
    out.update(read_slice(bytes, cursor, 24)?);
    Some(())
}
