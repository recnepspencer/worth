//! Deterministic, restart-safe auto idempotency key derivation rooted purely
//! in canonical durable branch and commit identity components (never process
//! owner ordinals, live incarnation fences, or wall-clock samples).
use sha2::{Digest, Sha256};
use worth_foundational::facade::ExecutionRequestPolicy;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::{WorthQueryAdvancementDenial, WorthQueryAdvancementPhase},
    primary_graph::{
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryProductBranchAdmissionDenial,
    },
    product::WorthQueryProductBranch,
};

/// Maximum deterministic probes when skipping already-occupied keys.
/// Bounded so cumulative probing inside one metered request exhausts cleanly
/// into `AutoIdempotencyKeysExhausted` within the default host work ceiling.
pub(crate) const MAX_AUTO_KEY_PROBES: u64 = 64;

/// Deterministic restart-safe auto idempotency key derivation for
/// [`WorthQueryHostSession::run`](super::WorthQueryHostSession::run).
///
/// Keys are derived from canonical durable branch and commit coordinates plus
/// a deterministic probe index, never from wall-clock samples or live process
/// owner/incarnation fences.
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

/// Computes the canonical 32-byte seed purely from durable branch/commit
/// components — never hashing process owner ordinals, live incarnation fences,
/// or `Debug` formatting.
pub(crate) fn durable_branch_seed_in_advancement<Schema: ApplicationSchema>(
    graph: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    phase: &WorthQueryAdvancementPhase<'_>,
) -> Result<[u8; 32], WorthQueryProductBranchAdmissionDenial> {
    let selected = graph.on_branch(branch).select(phase)?;
    let product = selected.product();
    let branch_name = product.branch_identity().name().as_str();
    let lifecycle_ordinal = product.lifecycle_incarnation().ordinal();
    let reference_generation = product.reference_generation().get();
    let commit_ordinal = product.selected_commit().ordinal();

    let mut hasher = Sha256::new();
    hasher.update(b"worth.durable-branch-seed.v2");
    hasher.update((branch_name.len() as u64).to_le_bytes());
    hasher.update(branch_name.as_bytes());
    hasher.update(lifecycle_ordinal.to_le_bytes());
    hasher.update(reference_generation.to_le_bytes());
    hasher.update(commit_ordinal.to_le_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    Ok(out)
}

/// Derives the canonical durable branch seed under the Home's installed
/// default execution policy (matching `WorthQueryHostSession`'s policy ceiling).
pub(crate) fn durable_branch_seed_for_installed<Schema: ApplicationSchema>(
    graph: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    scope: &WorthQueryRequestScope,
) -> Result<[u8; 32], DurableSeedDenial> {
    let policy = super::lease::default_session_policy(graph.installed_execution_policy());
    durable_branch_seed(graph, branch, policy, scope)
}

/// Derives the canonical durable branch seed under the caller's execution policy.
pub(crate) fn durable_branch_seed<Schema: ApplicationSchema>(
    graph: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: WorthQueryProductBranch,
    policy: ExecutionRequestPolicy,
    scope: &WorthQueryRequestScope,
) -> Result<[u8; 32], DurableSeedDenial> {
    super::lease::with_policy_advancement(graph, policy, scope, |phase| {
        durable_branch_seed_in_advancement(graph, branch, &phase)
            .map_err(DurableSeedDenial::Selection)
    })
    .map_err(DurableSeedDenial::Advancement)?
}
