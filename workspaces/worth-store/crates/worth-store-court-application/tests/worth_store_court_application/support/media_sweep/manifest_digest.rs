//! Deterministic SHA-256 digest covering every continuation-relevant section
//! inside a `WAPHCKP1` v3 `home.checkpoint` payload (Relational `WRTHRCS1` +
//! `WRTHCAI1`, World companion `WRWMREG1`, World `WWRLDCP1`, and Signal
//! `WSIGOWNR`).

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::section_digests::{verify_and_hash_world, verify_and_hash_wrthcai1, Wsrcai01Bindings};
use super::signal_digest::verify_and_hash_signal;

pub(crate) struct CheckpointDigestSummary {
    pub(crate) digest: [u8; 32],
    pub(crate) home: [u8; 16],
    pub(crate) world_through: [u64; 3],
    pub(crate) signal_covered: [u64; 2],
    pub(crate) world_record_digests: [BTreeMap<u64, [u8; 32]>; 2],
}

/// Verify the full `WAPHCKP1` v3 checkpoint envelope — Relational (`WSRCAI01`
/// -> `WRTHRCS1` + `WRTHCAI1` + `WRWMREG1`), World (`WWRLDCP1`), and Signal
/// (`WSIGOWNR`) — and hash all deterministic bytes across all three sections,
/// normalizing only per-home `HomeIdentity` and per-runtime
/// `runtime_instance_id` values. Any malformed frame, checksum mismatch,
/// typed cut binding mismatch, or unsupported non-empty World table fails
/// closed (`None`) so World continuation executes unconditionally.
pub(crate) fn relational_manifest_digest(bytes: &[u8]) -> Option<CheckpointDigestSummary> {
    let (native, world, signal) = parse_waphckp1_sections(bytes)?;
    let (header_cut, bindings, source, image, companion) = parse_wsrcai01(native)?;
    let (source_cut, cp, init_and_orig) = parse_wrthrcs1_checkpoint(source)?;
    if !cp.starts_with(b"WORTH-REL-CHK25\n") {
        return None;
    }
    verify_wrwmreg1(companion, &bindings)?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes[14..16]);
    hasher.update(&bytes[24..26]);
    hasher.update(&bytes[34..36]);
    hasher.update(header_cut);
    hasher.update(source_cut);
    hash_normalized_relational_bytes(cp, &mut hasher).filter(|&n| n > 0)?;
    hasher.update((init_and_orig.len() as u64).to_le_bytes());
    hasher.update(init_and_orig);
    verify_and_hash_wrthcai1(image, &bindings, &mut hasher)?;
    let (sig_digests, signal_covered) = verify_and_hash_signal(signal, bindings.home, &mut hasher)?;
    let world_record_digests = verify_and_hash_world(world, &bindings, &sig_digests, &mut hasher)?;
    let home: [u8; 16] = bindings.home.try_into().ok()?;
    let world_through = [
        u64::from_le_bytes(bindings.world_through[0..8].try_into().ok()?),
        u64::from_le_bytes(bindings.world_through[8..16].try_into().ok()?),
        u64::from_le_bytes(bindings.world_through[16..24].try_into().ok()?),
    ];
    Some(CheckpointDigestSummary {
        digest: hasher.finalize().into(),
        home,
        world_through,
        signal_covered,
        world_record_digests,
    })
}

fn parse_waphckp1_sections(bytes: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    if bytes.len() < 44 || &bytes[..8] != b"WAPHCKP1" {
        return None;
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    let count = u16::from_le_bytes(bytes[12..14].try_into().ok()?);
    if version != 3 || count != 3 {
        return None;
    }
    let native_len = u64_le(bytes, 16)?;
    let world_len = u64_le(bytes, 26)?;
    let signal_len = u64_le(bytes, 36)?;
    let native_end = 44usize.checked_add(native_len)?;
    let world_end = native_end.checked_add(world_len)?;
    let total = world_end.checked_add(signal_len)?;
    if native_len == 0 || world_len == 0 || signal_len == 0 || total != bytes.len() {
        return None;
    }
    Some((
        &bytes[44..native_end],
        &bytes[native_end..world_end],
        &bytes[world_end..total],
    ))
}

fn parse_wsrcai01(native: &[u8]) -> Option<([u8; 80], Wsrcai01Bindings<'_>, &[u8], &[u8], &[u8])> {
    const HEADER: usize = 292;
    if native.len() < HEADER || &native[..8] != b"WSRCAI01" {
        return None;
    }
    if u32::from_le_bytes(native[8..12].try_into().ok()?) != 1 {
        return None;
    }
    let mut cursor = HEADER;
    let mut ranges = [0..0, 0..0, 0..0];
    for (index, slot) in ranges.iter_mut().enumerate() {
        let len = u64_le(native, 172 + index * 8)?;
        let end = cursor.checked_add(len)?;
        let child = native.get(cursor..end)?;
        let expected = &native[196 + index * 32..196 + (index + 1) * 32];
        if Sha256::digest(child).as_slice() != expected {
            return None;
        }
        *slot = cursor..end;
        cursor = end;
    }
    if cursor != native.len() {
        return None;
    }
    // Deterministic WSRCAI01 cut fields: covered/transitions/initial (24B),
    // native_origin (32B), world_through (24B); excludes per-home UUID [68..84].
    let mut cut = [0u8; 80];
    cut[..56].copy_from_slice(&native[12..68]);
    cut[56..80].copy_from_slice(&native[84..108]);
    let bindings = Wsrcai01Bindings {
        home: &native[68..84],
        covered: &native[12..20],
        covered_transitions: &native[20..28],
        native_origin: &native[36..68],
        world_through: &native[84..108],
        checkpoint_digest: &native[108..140],
        world_image_digest: &native[140..172],
        source_digest: &native[196..228],
        image_digest: &native[228..260],
    };
    Some((
        cut,
        bindings,
        &native[ranges[0].clone()],
        &native[ranges[1].clone()],
        &native[ranges[2].clone()],
    ))
}

fn parse_wrthrcs1_checkpoint(source: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    const HEADER: usize = 84;
    if source.len() < HEADER || &source[..8] != b"WRTHRCS1" {
        return None;
    }
    if u32::from_le_bytes(source[8..12].try_into().ok()?) != 2 {
        return None;
    }
    let cp_end = HEADER.checked_add(u64_le(source, 28)?)?;
    let init_end = cp_end.checked_add(u64_le(source, 36)?)?;
    let end = init_end.checked_add(u64_le(source, 44)?)?;
    if end != source.len() {
        return None;
    }
    let mut digest = Sha256::new();
    digest.update(&source[..52]);
    digest.update(&source[HEADER..end]);
    if digest.finalize().as_slice() != &source[52..84] {
        return None;
    }
    Some((
        &source[12..28],
        &source[HEADER..cp_end],
        &source[cp_end..end],
    ))
}

pub(super) fn hash_normalized_relational_bytes(cp: &[u8], hasher: &mut Sha256) -> Option<usize> {
    const ID_KEY: &[u8] = b"\xb3runtime_instance_id";
    const REL_PREFIX: &[u8] = b"relational/";
    let mut cursor = 0usize;
    let mut normalized = 0usize;
    while cursor < cp.len() {
        let next_id = find_bytes(&cp[cursor..], ID_KEY).map(|o| cursor + o);
        let next_rel = find_bytes(&cp[cursor..], REL_PREFIX).map(|o| cursor + o);
        match (next_id, next_rel) {
            (Some(id_pos), Some(rel_pos)) if id_pos < rel_pos => {
                let end_key = id_pos + ID_KEY.len();
                let int_len = msgpack_uint_len(cp.get(end_key..)?)?;
                hasher.update(&cp[cursor..end_key]);
                hasher.update(b"<id>");
                cursor = end_key + int_len;
                normalized += 1;
            }
            (Some(id_pos), None) => {
                let end_key = id_pos + ID_KEY.len();
                let int_len = msgpack_uint_len(cp.get(end_key..)?)?;
                hasher.update(&cp[cursor..end_key]);
                hasher.update(b"<id>");
                cursor = end_key + int_len;
                normalized += 1;
            }
            (_, Some(rel_pos)) => {
                if let Some((str_hdr_start, after_slash, suffix_len)) =
                    relational_runtime_tag_span(cp, rel_pos, cursor)
                {
                    hasher.update(&cp[cursor..str_hdr_start]);
                    hasher.update(b"<rel-id>/");
                    hasher.update((suffix_len as u64).to_le_bytes());
                    cursor = after_slash;
                    normalized += 1;
                } else {
                    let advance = rel_pos + REL_PREFIX.len();
                    hasher.update(&cp[cursor..advance]);
                    cursor = advance;
                }
            }
            (None, None) => {
                hasher.update(&cp[cursor..]);
                cursor = cp.len();
            }
        }
    }
    Some(normalized)
}

fn relational_runtime_tag_span(
    cp: &[u8],
    rel_pos: usize,
    min_start: usize,
) -> Option<(usize, usize, usize)> {
    let digits_start = rel_pos + b"relational/".len();
    let mut end = digits_start;
    while end < cp.len() && cp[end].is_ascii_digit() {
        end += 1;
    }
    if end == digits_start || end - digits_start > 20 || cp.get(end) != Some(&b'/') {
        return None;
    }
    let tag_len = end + 1 - rel_pos;
    let (hdr_start, str_len) = if rel_pos >= min_start + 2
        && cp[rel_pos - 2] == 0xd9
        && usize::from(cp[rel_pos - 1]) >= 32
    {
        (rel_pos - 2, usize::from(cp[rel_pos - 1]))
    } else if rel_pos > min_start && (0xa0..=0xbf).contains(&cp[rel_pos - 1]) {
        (rel_pos - 1, usize::from(cp[rel_pos - 1] - 0xa0))
    } else {
        return None;
    };
    if str_len < tag_len || rel_pos + str_len > cp.len() {
        return None;
    }
    Some((hdr_start, end + 1, str_len - tag_len))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

pub(super) fn u64_le(bytes: &[u8], offset: usize) -> Option<usize> {
    usize::try_from(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
    .ok()
}

fn msgpack_uint_len(bytes: &[u8]) -> Option<usize> {
    let len = match *bytes.first()? {
        0x00..=0x7f => 1,
        0xcc => 2,
        0xcd => 3,
        0xce => 5,
        0xcf => 9,
        _ => return None,
    };
    (bytes.len() >= len).then_some(len)
}

fn verify_wrwmreg1(companion: &[u8], b: &Wsrcai01Bindings<'_>) -> Option<()> {
    const PAYLOAD: usize = 229;
    if companion.len() != PAYLOAD + 32 || &companion[..8] != b"WRWMREG1" {
        return None;
    }
    if u32::from_le_bytes(companion[8..12].try_into().ok()?) != 1 || companion[12] != 1 {
        return None;
    }
    if &companion[13..29] != b.home
        || &companion[29..37] != b.covered
        || &companion[37..45] != b.covered_transitions
        || &companion[45..77] != b.checkpoint_digest
        || &companion[77..109] != b.native_origin
        || &companion[109..141] != b.source_digest
        || &companion[141..165] != b.world_through
        || &companion[165..197] != b.world_image_digest
        || &companion[197..229] != b.image_digest
    {
        return None;
    }
    let mut h = Sha256::new();
    h.update(b"worth.world.receipt-registry.home.v1\0");
    h.update(&companion[..PAYLOAD]);
    (h.finalize().as_slice() == &companion[PAYLOAD..]).then_some(())
}

pub(crate) fn encode_canonical_home_cut(b: &Wsrcai01Bindings<'_>) -> Vec<u8> {
    let mut out = Vec::with_capacity(320);
    out.push(0x84);
    out.extend_from_slice(b"\xa5scope");
    out.push(0x81);
    out.extend_from_slice(b"\xa4Home");
    out.push(0x85);
    out.extend_from_slice(b"\xa4home");
    out.extend_from_slice(&[0xc4, 16]);
    out.extend_from_slice(b.home);
    out.extend_from_slice(b"\xadnative_origin");
    out.extend_from_slice(&[0xc4, 32]);
    out.extend_from_slice(b.native_origin);
    out.extend_from_slice(b"\xb6complete_source_digest");
    out.extend_from_slice(&[0xc4, 32]);
    out.extend_from_slice(b.source_digest);
    out.extend_from_slice(b"\xadworld_through");
    out.push(0x93);
    for i in 0..3 {
        let v = u64::from_le_bytes(b.world_through[i * 8..(i + 1) * 8].try_into().unwrap());
        push_mp_u64(&mut out, v);
    }
    out.extend_from_slice(b"\xb2world_image_digest");
    out.extend_from_slice(&[0xc4, 32]);
    out.extend_from_slice(b.world_image_digest);
    out.extend_from_slice(b"\xb1canonical_through");
    push_mp_u64(&mut out, u64::from_le_bytes(b.covered.try_into().unwrap()));
    out.extend_from_slice(b"\xb3transitions_through");
    push_mp_u64(
        &mut out,
        u64::from_le_bytes(b.covered_transitions.try_into().unwrap()),
    );
    out.extend_from_slice(b"\xb1checkpoint_digest");
    out.extend_from_slice(&[0xc4, 32]);
    out.extend_from_slice(b.checkpoint_digest);
    out
}

fn push_mp_u64(out: &mut Vec<u8>, value: u64) {
    if let Ok(v) = u8::try_from(value) {
        if v <= 0x7f {
            out.push(v);
        } else {
            out.push(0xcc);
            out.push(v);
        }
    } else if let Ok(v) = u16::try_from(value) {
        out.push(0xcd);
        out.extend_from_slice(&v.to_be_bytes());
    } else if let Ok(v) = u32::try_from(value) {
        out.push(0xce);
        out.extend_from_slice(&v.to_be_bytes());
    } else {
        out.push(0xcf);
        out.extend_from_slice(&value.to_be_bytes());
    }
}
