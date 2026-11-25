//! Deterministic, restart-safe auto idempotency key derivation rooted in
//! durable branch state rather than wall-clock timestamps.
use sha2::{Digest, Sha256};
use worth_foundational::facade::ExecutionRequestPolicy;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProductBranchAdmissionDenial,
    },
    product::WorthQueryProductBranch,
};

/// Maximum deterministic probes when skipping already-occupied keys.
pub(crate) const MAX_AUTO_KEY_PROBES: u64 = 1024;

/// Deterministic restart-safe auto idempotency key derivation for
/// [`WorthQueryHostSession::run`](super::WorthQueryHostSession::run).
///
/// Keys are derived from the target branch's durable committed head identity
/// plus a deterministic probe index, never from wall-clock samples, and
/// callers skip any already-occupied key cleanly without side effects.
pub trait WorthQueryAutoIdempotencyKey: Sized {
    fn derive_auto_key(seed: [u8; 32], probe: u64) -> Self;
}

fn hash_seed_and_probe(seed: [u8; 32], domain: &[u8], probe: u64) -> (u64, u64) {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(seed);
    hasher.update(probe.to_le_bytes());
    let digest = hasher.finalize();
    let mut hi_bytes = [0u8; 8];
    let mut lo_bytes = [0u8; 8];
    hi_bytes.copy_from_slice(&digest[0..8]);
    lo_bytes.copy_from_slice(&digest[8..16]);
    (u64::from_le_bytes(hi_bytes), u64::from_le_bytes(lo_bytes))
}

impl WorthQueryAutoIdempotencyKey for u64 {
    fn derive_auto_key(seed: [u8; 32], probe: u64) -> Self {
        let (hi, _) = hash_seed_and_probe(seed, b"worth.auto-key.u64.v1", probe);
        hi.max(1)
    }
}

impl WorthQueryAutoIdempotencyKey for i64 {
    fn derive_auto_key(seed: [u8; 32], probe: u64) -> Self {
        let (hi, _) = hash_seed_and_probe(seed, b"worth.auto-key.i64.v1", probe);
        ((hi & (i64::MAX as u64)).max(1)) as i64
    }
}

impl WorthQueryAutoIdempotencyKey for u128 {
    fn derive_auto_key(seed: [u8; 32], probe: u64) -> Self {
        let (hi, lo) = hash_seed_and_probe(seed, b"worth.auto-key.u128.v1", probe);
        ((u128::from(hi) << 64) | u128::from(lo)).max(1)
    }
}

impl WorthQueryAutoIdempotencyKey for String {
    fn derive_auto_key(seed: [u8; 32], probe: u64) -> Self {
        let (hi, lo) = hash_seed_and_probe(seed, b"worth.auto-key.str.v1", probe);
        format!("worth-auto-{hi:016x}-{lo:016x}")
    }
}

/// Derives a 63-bit decimal integer correction key from durable branch state
/// and a deterministic probe index. Decimal 63-bit strings parse cleanly for
/// `u64`, `i64`, `u128`, and `String` action idempotency keys alike.
pub(crate) fn derive_decimal_correction_key(seed: [u8; 32], tag: &[u8], probe: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"worth.auto-correction-key.v1");
    hasher.update(tag);
    hasher.update(seed);
    hasher.update(probe.to_le_bytes());
    let digest = hasher.finalize();
    let mut hi_bytes = [0u8; 8];
    hi_bytes.copy_from_slice(&digest[0..8]);
    let value = (u64::from_le_bytes(hi_bytes) & (i64::MAX as u64)).max(1);
    value.to_string()
}

pub(crate) enum DurableSeedDenial {
    Advancement(WorthQueryAdvancementDenial),
    Selection(WorthQueryProductBranchAdmissionDenial),
}

/// Derives a deterministic 32-byte seed from the target branch's durable
/// committed head identity under the caller's execution policy.
pub(crate) fn durable_branch_seed<Schema: ApplicationSchema>(
    graph: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    policy: ExecutionRequestPolicy,
    scope: &WorthQueryRequestScope,
) -> Result<[u8; 32], DurableSeedDenial> {
    let selected_digest = super::lease::with_policy_advancement(graph, policy, scope, |phase| {
        let selected = graph
            .on_branch(branch)
            .select(&phase)
            .map_err(DurableSeedDenial::Selection)?;
        let mut hasher = Sha256::new();
        hasher.update(b"worth.durable-branch-seed.v1");
        hasher.update(format!("{:?}", selected.product().branch_identity()).as_bytes());
        hasher.update(format!("{:?}", selected.product().selected_commit()).as_bytes());
        let digest = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        Ok(out)
    })
    .map_err(DurableSeedDenial::Advancement)?;
    selected_digest
}
