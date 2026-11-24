//! Cryptographic verification and deterministic hashing of Signal (`WSIGOWNR`
//! v1 kind 5) and nested `WSIGOWNR` / `WSIGAUTH` / `WSIGCONF` frames inside
//! `WAPHCKP1` v3 `home.checkpoint` payloads.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{OrderedJournalName, ServingPhysicalRuntime};

use super::manifest_digest::{u64_le, CheckpointDigestSummary};
use super::section_digests::verify_world_journal_entry;

pub(super) fn verify_and_hash_signal(
    signal: &[u8],
    home: &[u8],
    out: &mut Sha256,
) -> Option<(Vec<[u8; 32]>, [u64; 2])> {
    let p = verify_56b_frame(signal, b"WSIGOWNR")?;
    let version = u32::from_le_bytes(signal[8..12].try_into().ok()?);
    let kind = u32::from_le_bytes(signal[12..16].try_into().ok()?);
    if version != 1 || kind != 5 {
        return None;
    }
    let mut c = 0usize;
    if read_slice(p, &mut c, 16)? != home {
        return None;
    }
    out.update(b"WSIGOWNR");
    out.update(&signal[8..16]);
    // slot(8) + 6 frontier counters(48)
    out.update(read_slice(p, &mut c, 56)?);
    let snap_tag = *read_slice(p, &mut c, 1)?.first()?;
    out.update([snap_tag]);
    match snap_tag {
        0 => {}
        1 => out.update(read_slice(p, &mut c, 8)?),
        _ => return None,
    }
    let branch_nodes = read_u64(p, &mut c)?;
    out.update((branch_nodes as u64).to_le_bytes());
    for _ in 0..branch_nodes {
        if read_slice(p, &mut c, 16)? != home {
            return None;
        }
        out.update(read_slice(p, &mut c, 16)?);
        let node_count = read_u64(p, &mut c)?;
        out.update((node_count as u64).to_le_bytes());
        out.update(read_slice(p, &mut c, node_count.checked_mul(16)?)?);
    }
    let mut sig_digests: Vec<[u8; 32]> = Vec::new();
    let mut covered = [0u64; 2];
    for slot in &mut covered {
        let count = read_u64(p, &mut c)?;
        *slot = count as u64;
        out.update((*slot).to_le_bytes());
        for _ in 0..count {
            let blob_len = read_u64(p, &mut c)?;
            let blob = read_slice(p, &mut c, blob_len)?;
            let digest = verify_wsigownr_inner(blob, home, &sig_digests, out)?;
            sig_digests.push(digest);
        }
    }
    (c == p.len()).then_some((sig_digests, covered))
}

/// Cryptographically verify and hash every entry across all non-Relational
/// Application journals (`signal.definitions`, `signal.advances`,
/// `world.branches`, `world.custody`, `world.occurrences`, `home.transitions`,
/// `query.*`). Fails closed (`None`) if any journal is unreadable, any
/// uncovered Signal/World frame fails verification, or any `world.occurrences` /
/// `home.transitions` / `query.*` entry exists.
pub(crate) fn verify_application_journals(
    serving: &ServingPhysicalRuntime,
    checkpoint: Option<&CheckpointDigestSummary>,
) -> Option<[u8; 32]> {
    for empty_name in [
        "world.occurrences",
        "home.transitions",
        "query.obligations",
        "query.dispatch",
        "query.corrections",
    ] {
        let name = OrderedJournalName::new(empty_name).ok()?;
        if serving.ordered_journal(name).ok()?.head().ok()?.is_some() {
            return None;
        }
    }
    if let Some(cp) = checkpoint
        && cp.world_through[2] != 0
    {
        return None;
    }

    let mut home = checkpoint.map(|cp| cp.home);
    let mut sig_digests = Vec::new();
    let mut out = Sha256::new();
    for (idx, jname) in ["signal.definitions", "signal.advances"]
        .into_iter()
        .enumerate()
    {
        let journal = serving
            .ordered_journal(OrderedJournalName::new(jname).ok()?)
            .ok()?;
        let head = journal.head().ok()?.map_or(0, |e| e.position());
        if let Some(cp) = checkpoint
            && head < cp.signal_covered[idx]
        {
            return None;
        }
        out.update(jname.as_bytes());
        out.update(head.to_le_bytes());
        for pos in 1..=head {
            let payload = journal.read(pos, 1 << 30).ok()?;
            let raw = payload.bytes();
            if home.is_none() {
                if raw.len() < 72 || &raw[..8] != b"WSIGOWNR" {
                    return None;
                }
                home = Some(raw[56..72].try_into().ok()?);
            }
            let h = home?;
            out.update(pos.to_le_bytes());
            let digest = verify_wsigownr_inner(raw, &h, &sig_digests, &mut out)?;
            sig_digests.push(digest);
        }
    }
    for (idx, jname) in ["world.branches", "world.custody"].into_iter().enumerate() {
        let journal = serving
            .ordered_journal(OrderedJournalName::new(jname).ok()?)
            .ok()?;
        let head = journal.head().ok()?.map_or(0, |e| e.position());
        if let Some(cp) = checkpoint
            && head < cp.world_through[idx]
        {
            return None;
        }
        out.update(jname.as_bytes());
        out.update(head.to_le_bytes());
        for pos in 1..=head {
            let payload = journal.read(pos, 1 << 30).ok()?;
            let h = home?;
            verify_world_journal_entry_against_checkpoint(
                checkpoint,
                idx,
                pos,
                payload.bytes(),
                &h,
                &sig_digests,
                &mut out,
            )?;
        }
    }
    Some(out.finalize().into())
}

fn verify_world_journal_entry_against_checkpoint(
    checkpoint: Option<&CheckpointDigestSummary>,
    journal_idx: usize,
    pos: u64,
    raw: &[u8],
    home: &[u8],
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) -> Option<()> {
    if let Some(cp) = checkpoint
        && pos <= cp.world_through[journal_idx]
    {
        let expected = cp.world_record_digests[journal_idx].get(&pos)?;
        if Sha256::digest(raw).as_slice() != expected.as_slice() {
            return None;
        }
    }
    out.update(pos.to_le_bytes());
    verify_world_journal_entry(raw, home, sig_digests, out)
}

pub(super) fn verify_wsigownr_inner(
    frame: &[u8],
    home: &[u8],
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) -> Option<[u8; 32]> {
    let payload = verify_56b_frame(frame, b"WSIGOWNR")?;
    if u32::from_le_bytes(frame[8..12].try_into().ok()?) != 1 {
        return None;
    }
    let kind = u32::from_le_bytes(frame[12..16].try_into().ok()?);
    out.update(&frame[8..24]);
    if kind == 1 {
        verify_signal_definition_payload(payload, home, sig_digests, out)?;
    } else {
        hash_replacing_identities(payload, home, sig_digests, out);
    }
    Some(Sha256::digest(frame).into())
}

fn verify_56b_frame<'a>(frame: &'a [u8], magic: &[u8; 8]) -> Option<&'a [u8]> {
    const HEADER: usize = 56;
    if frame.len() < HEADER || &frame[..8] != magic {
        return None;
    }
    let len = u64_le(frame, 16)?;
    if HEADER.checked_add(len)? != frame.len() {
        return None;
    }
    let mut h = Sha256::new();
    h.update(&frame[8..24]);
    h.update(&frame[HEADER..]);
    (h.finalize().as_slice() == &frame[24..HEADER]).then_some(&frame[HEADER..])
}

fn verify_signal_definition_payload(
    p: &[u8],
    home: &[u8],
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) -> Option<()> {
    let mut c = 0usize;
    if read_slice(p, &mut c, 16)? != home {
        return None;
    }
    out.update(read_slice(p, &mut c, 16)?);
    if read_slice(p, &mut c, 16)? != home {
        return None;
    }
    out.update(read_slice(p, &mut c, 16)?);
    let has_pred = *read_slice(p, &mut c, 1)?.first()?;
    out.update([has_pred]);
    match has_pred {
        0 => {}
        1 => hash_replacing_identities(read_slice(p, &mut c, 64)?, home, sig_digests, out),
        _ => return None,
    }
    for magic in [b"WSIGAUTH", b"WSIGCONF"] {
        let len = read_u64(p, &mut c)?;
        let blob = read_slice(p, &mut c, len)?;
        let inner = verify_56b_frame(blob, magic)?;
        out.update(&blob[..24]);
        hash_replacing_identities(inner, home, sig_digests, out);
    }
    let cond_count = read_u64(p, &mut c)?;
    out.update((cond_count as u64).to_le_bytes());
    let cond_bytes = read_slice(p, &mut c, cond_count.checked_mul(88)?)?;
    hash_replacing_identities(cond_bytes, home, sig_digests, out);
    let bridge_len = read_u64(p, &mut c)?;
    let bridge = read_slice(p, &mut c, bridge_len)?;
    if bridge.len() >= 56 && u64_le(bridge, 16) == Some(bridge.len() - 56) {
        let magic: [u8; 8] = bridge[..8].try_into().ok()?;
        if let Some(inner) = verify_56b_frame(bridge, &magic) {
            out.update(&bridge[..24]);
            hash_replacing_identities(inner, home, sig_digests, out);
            return (c == p.len()).then_some(());
        }
    }
    hash_replacing_identities(bridge, home, sig_digests, out);
    (c == p.len()).then_some(())
}

pub(super) fn read_slice<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Option<&'a [u8]> {
    let end = cursor.checked_add(len)?;
    let slice = bytes.get(*cursor..end)?;
    *cursor = end;
    Some(slice)
}

pub(super) fn read_u64(bytes: &[u8], cursor: &mut usize) -> Option<usize> {
    let raw = read_slice(bytes, cursor, 8)?;
    usize::try_from(u64::from_le_bytes(raw.try_into().ok()?)).ok()
}

pub(super) fn hash_replacing_identities(
    bytes: &[u8],
    home: &[u8],
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) {
    out.update((bytes.len() as u64).to_le_bytes());
    let mut i = 0usize;
    let mut emitted = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(home) {
            out.update(&bytes[emitted..i]);
            out.update(b"<home>");
            i += home.len();
            emitted = i;
        } else if let Some(idx) = sig_digests.iter().position(|d| bytes[i..].starts_with(d)) {
            out.update(&bytes[emitted..i]);
            out.update(b"<sig-d>");
            out.update((idx as u64).to_le_bytes());
            i += 32;
            emitted = i;
        } else {
            i += 1;
        }
    }
    out.update(&bytes[emitted..]);
}

pub(super) fn hash_normalized_json(
    bytes: &[u8],
    home: &[u8],
    sig_digests: &[[u8; 32]],
    out: &mut Sha256,
) {
    let hex = |raw: &[u8]| {
        let mut s = Vec::with_capacity(raw.len() * 2);
        for &b in raw {
            s.push(b"0123456789abcdef"[(b >> 4) as usize]);
            s.push(b"0123456789abcdef"[(b & 0x0f) as usize]);
        }
        s
    };
    let home_hex = hex(home);
    let sig_hex: Vec<Vec<u8>> = sig_digests.iter().map(|d| hex(d)).collect();
    let mut i = 0usize;
    let mut emitted = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"\"runtime_instance_id\":") {
            let start = i + b"\"runtime_instance_id\":".len();
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start {
                out.update(&bytes[emitted..start]);
                out.update(b"<id>");
                i = end;
                emitted = i;
                continue;
            }
        }
        if bytes[i..].starts_with(b"relational/") {
            let start = i + b"relational/".len();
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start && bytes.get(end) == Some(&b'/') {
                out.update(&bytes[emitted..start]);
                out.update(b"<id>/");
                i = end + 1;
                emitted = i;
                continue;
            }
        }
        if bytes[i..].starts_with(&home_hex) {
            out.update(&bytes[emitted..i]);
            out.update(b"<home>");
            i += home_hex.len();
            emitted = i;
        } else if let Some(idx) = sig_hex.iter().position(|d| bytes[i..].starts_with(d)) {
            out.update(&bytes[emitted..i]);
            out.update(b"<sig-d>");
            out.update((idx as u64).to_le_bytes());
            i += 64;
            emitted = i;
        } else {
            i += 1;
        }
    }
    out.update(&bytes[emitted..]);
    out.update(b"\0/json");
}
