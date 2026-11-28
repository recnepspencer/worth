//! Strictly crate-private Home-scoped authority for minting Home-MAC-authenticated
//! [`RetainedBasisToken`]s, sealing fixed-size query cursor positions under the
//! Home's persisted secret key, minting reclaim-fenced [`ReaderLoan`]s over
//! concrete Store custody, and admitting [`HistoricalCursor`]s.
//!
//! Because this authority is `pub(crate)` inside `worth-store-application` and
//! has zero public constructors or witness parameters, safe external Rust in
//! any other crate cannot name, mint, or forge an authority, loan, sealed
//! cursor position, or cursor.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequest;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_runtime_world::facade::{
    CanonicalCursorKey, HistoricalReadDenial, RetainedBasisTokenFields,
};

use super::cursor::{
    ClosureCursorStream, HistoricalCursor, OwnedLoanGuard, ReaderLoan, StoreLoanCustody,
};
use super::token::RetainedBasisToken;

const POS_SEAL_MAGIC: &[u8; 4] = b"WHQP";
const POS_NONCE_BYTES: usize = 16;
const POS_PLAINTEXT_BYTES: usize = 8 + 4 + 8 + 4;
const POS_TAG_BYTES: usize = 32;
pub(super) const SEALED_QUERY_POSITION_BYTES: usize =
    POS_SEAL_MAGIC.len() + POS_NONCE_BYTES + POS_PLAINTEXT_BYTES + POS_TAG_BYTES;

const POS_ENC_KEY_DOMAIN: &[u8] = b"worth.store.query-cursor-pos.enc.v1";
const POS_MAC_KEY_DOMAIN: &[u8] = b"worth.store.query-cursor-pos.mac.v1";
const POS_TAG_DOMAIN: &[u8] = b"worth.store.query-cursor-pos.aead.v1";
const POS_NONCE_DOMAIN: &[u8] = b"worth.store.query-cursor-pos.nonce.v1";

static POS_NONCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Home-scoped crate-private capability for minting, verifying, and advancing
/// [`RetainedBasisToken`]s, sealing query cursor positions, and admitting
/// reclaim-fenced [`HistoricalCursor`]s.
#[derive(Clone)]
pub(crate) struct HistoricalCursorAuthority {
    mac_key: [u8; 32],
}

impl std::fmt::Debug for HistoricalCursorAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoricalCursorAuthority")
            .field("mac_key", &"<redacted>")
            .finish()
    }
}

impl HistoricalCursorAuthority {
    /// Crate-private constructor bound directly to a Home's persisted MAC key.
    pub(super) const fn from_home_key(mac_key: [u8; 32]) -> Self {
        Self { mac_key }
    }

    /// Encrypts and authenticates a fixed-size `(ordinal, root)` query cursor
    /// resume anchor under this Home's persisted secret key + cursor `aad`, so
    /// neither withheld ordering values nor raw `EntityId` bytes appear in
    /// plaintext in `RetainedBasisToken`, `canonical_key_position()`, or `Debug`.
    pub(super) fn seal_query_cursor_position(
        &self,
        ordinal: u64,
        root: EntityId,
        aad: &[u8],
    ) -> Vec<u8> {
        let mut plaintext = [0u8; POS_PLAINTEXT_BYTES];
        plaintext[0..8].copy_from_slice(&ordinal.to_be_bytes());
        plaintext[8..12].copy_from_slice(&root.partition_id.0.to_le_bytes());
        plaintext[12..20].copy_from_slice(&root.local_slot.0.to_le_bytes());
        plaintext[20..24].copy_from_slice(&root.generation.0.to_le_bytes());

        let enc_key = hmac_sha256(&self.mac_key, POS_ENC_KEY_DOMAIN, &[]);
        let mac_key = hmac_sha256(&self.mac_key, POS_MAC_KEY_DOMAIN, &[]);
        let nonce = fresh_position_nonce(&self.mac_key, aad);
        let mut ciphertext = plaintext;
        xor_sha256_ctr(&enc_key, &nonce, &mut ciphertext);
        let tag = compute_position_tag(&mac_key, aad, &nonce, &ciphertext);

        let mut out = Vec::with_capacity(SEALED_QUERY_POSITION_BYTES);
        out.extend_from_slice(POS_SEAL_MAGIC);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ciphertext);
        out.extend_from_slice(&tag);
        out
    }

    /// Authenticates and decrypts a fixed-size `(ordinal, root)` query cursor
    /// resume anchor produced by this Home's secret key under the same `aad`.
    pub(super) fn open_query_cursor_position(
        &self,
        sealed: &[u8],
        aad: &[u8],
    ) -> Option<(u64, EntityId)> {
        if sealed.len() != SEALED_QUERY_POSITION_BYTES || &sealed[..4] != POS_SEAL_MAGIC {
            return None;
        }
        let nonce: [u8; POS_NONCE_BYTES] = sealed[4..4 + POS_NONCE_BYTES].try_into().ok()?;
        let ct_start = 4 + POS_NONCE_BYTES;
        let ct_end = ct_start + POS_PLAINTEXT_BYTES;
        let ciphertext = &sealed[ct_start..ct_end];
        let tag: [u8; POS_TAG_BYTES] = sealed[ct_end..].try_into().ok()?;

        let enc_key = hmac_sha256(&self.mac_key, POS_ENC_KEY_DOMAIN, &[]);
        let mac_key = hmac_sha256(&self.mac_key, POS_MAC_KEY_DOMAIN, &[]);
        let expected = compute_position_tag(&mac_key, aad, &nonce, ciphertext);
        if !ct_eq_32(&tag, &expected) {
            return None;
        }
        let mut plaintext = [0u8; POS_PLAINTEXT_BYTES];
        plaintext.copy_from_slice(ciphertext);
        xor_sha256_ctr(&enc_key, &nonce, &mut plaintext);

        let ordinal = u64::from_be_bytes(plaintext[0..8].try_into().ok()?);
        let partition_id = PartitionId(u32::from_le_bytes(plaintext[8..12].try_into().ok()?));
        let local_slot = u64::from_le_bytes(plaintext[12..20].try_into().ok()?);
        let generation = u32::from_le_bytes(plaintext[20..24].try_into().ok()?);
        Some((ordinal, EntityId::new(partition_id, local_slot, generation)))
    }

    /// Derives a fresh unpredictable 256-bit per-page resume anchor nonce from
    /// this Home's persisted secret key so `WorthQueryApplicationRootWindowProgress`
    /// only exposes an opaque hash proof and never reveals the decrypted anchor `EntityId`.
    pub(super) fn derive_resume_anchor_nonce(&self, ordinal: u64, aad: &[u8]) -> [u8; 32] {
        let seq = POS_NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut h = RandomState::new().build_hasher();
        h.write_u64(seq);
        h.write_u64(ordinal);
        let mut hasher = Sha256::new();
        hasher.update(b"worth.store.query-cursor-pos.resume-anchor-nonce.v1");
        hasher.update(self.mac_key);
        hasher.update(ordinal.to_le_bytes());
        hasher.update(seq.to_le_bytes());
        hasher.update(h.finish().to_le_bytes());
        hasher.update((aad.len() as u64).to_le_bytes());
        hasher.update(aad);
        hasher.finalize().into()
    }

    /// Mints a Home-MAC-authenticated [`RetainedBasisToken`].
    pub(super) fn issue_token(
        &self,
        fields: RetainedBasisTokenFields,
    ) -> Option<RetainedBasisToken> {
        RetainedBasisToken::issue_with_key(fields, &self.mac_key)
    }

    /// Constant-time Home-MAC verification for a presented [`RetainedBasisToken`].
    pub(super) fn verify_token(&self, token: &RetainedBasisToken) -> bool {
        token.verify_mac(&self.mac_key)
    }

    /// Advances a verified token strictly monotonically under the Home's MAC key.
    pub(super) fn advance_token(
        &self,
        token: &RetainedBasisToken,
        next_position: CanonicalCursorKey,
    ) -> Option<RetainedBasisToken> {
        token.advance_with_key(next_position, &self.mac_key)
    }

    /// Admits a reclaim-fenced [`HistoricalCursor`] bound to a concrete
    /// Store-owned [`StoreLoanCustody`] and a Store-owned bounded page stream.
    pub(super) fn admit_cursor<'a, Row: 'a>(
        &self,
        token: RetainedBasisToken,
        loan_custody: StoreLoanCustody<'a>,
        loan_active: Arc<AtomicBool>,
        read_page: Box<
            dyn FnMut(
                    &ExecutionRequest<'_, '_>,
                    &CanonicalCursorKey,
                    usize,
                )
                    -> Result<(Vec<(CanonicalCursorKey, Row)>, bool), HistoricalReadDenial>
                + Send
                + 'a,
        >,
    ) -> HistoricalCursor<'a, Row> {
        let loan = ReaderLoan::from_guard(Box::new(OwnedLoanGuard::new(loan_custody, loan_active)));
        let stream = Box::new(ClosureCursorStream::new(read_page));
        HistoricalCursor::from_owner(loan, token, self.clone(), stream)
    }
}

fn fresh_position_nonce(key: &[u8; 32], aad: &[u8]) -> [u8; POS_NONCE_BYTES] {
    let seq = POS_NONCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut h1 = RandomState::new().build_hasher();
    h1.write_u64(seq);
    let mut h2 = RandomState::new().build_hasher();
    h2.write(aad);
    let mut hasher = Sha256::new();
    hasher.update(POS_NONCE_DOMAIN);
    hasher.update(key);
    hasher.update(seq.to_le_bytes());
    hasher.update(h1.finish().to_le_bytes());
    hasher.update(h2.finish().to_le_bytes());
    hasher.update((aad.len() as u64).to_le_bytes());
    hasher.update(aad);
    let digest: [u8; 32] = hasher.finalize().into();
    digest[..POS_NONCE_BYTES].try_into().expect("16-byte nonce")
}

fn xor_sha256_ctr(enc_key: &[u8; 32], nonce: &[u8; POS_NONCE_BYTES], buf: &mut [u8]) {
    for (block_idx, chunk) in buf.chunks_mut(32).enumerate() {
        let mut hasher = Sha256::new();
        hasher.update(enc_key);
        hasher.update(nonce);
        hasher.update((block_idx as u32).to_le_bytes());
        let stream: [u8; 32] = hasher.finalize().into();
        for (dst, mask) in chunk.iter_mut().zip(stream.iter()) {
            *dst ^= *mask;
        }
    }
}

fn compute_position_tag(
    mac_key: &[u8; 32],
    aad: &[u8],
    nonce: &[u8; POS_NONCE_BYTES],
    ciphertext: &[u8],
) -> [u8; 32] {
    let mut body = Vec::with_capacity(8 + aad.len() + POS_NONCE_BYTES + ciphertext.len());
    body.extend_from_slice(&(aad.len() as u64).to_le_bytes());
    body.extend_from_slice(aad);
    body.extend_from_slice(nonce);
    body.extend_from_slice(ciphertext);
    hmac_sha256(mac_key, POS_TAG_DOMAIN, &body)
}

fn hmac_sha256(key: &[u8; 32], domain: &[u8], body: &[u8]) -> [u8; 32] {
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5Cu8; 64];
    for i in 0..32 {
        ipad[i] ^= key[i];
        opad[i] ^= key[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(domain);
    inner.update(body);
    let inner_digest: [u8; 32] = inner.finalize().into();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    outer.finalize().into()
}

fn ct_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Generates a fresh unpredictable 256-bit per-Home MAC key.
pub(crate) fn generate_home_mac_key(home_identity: &[u8; 16]) -> [u8; 32] {
    use std::time::{SystemTime, UNIX_EPOCH};

    let (mut h1, mut h2) = (
        RandomState::new().build_hasher(),
        RandomState::new().build_hasher(),
    );
    h1.write(home_identity);
    h2.write_u32(std::process::id());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0u128, |d| d.as_nanos());
    let mut hasher = Sha256::new();
    hasher.update(b"worth.historical.home.mac-key.v1");
    hasher.update(home_identity);
    hasher.update(h1.finish().to_le_bytes());
    hasher.update(h2.finish().to_le_bytes());
    hasher.update(now.to_le_bytes());
    hasher.finalize().into()
}
