//! Single-harness 64-seed Differential DST for Shared Access over real Directory Homes
//! and real loopback TCP authenticated remote servers.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
    },
    shared_access::{
        AuthenticationRefusalReason, AuthorizationRefusalReason, BudgetRefusalReason,
        ProtocolRefusalReason, RemoteAuthConfig, RemoteAuthKey, RemoteAuthToken,
        RemoteTokenIssueRequest, ScopeAccess, SharedAccessDenial, WatchRefusalReason,
    },
    WorthQueryApplicationHome,
};

use super::dst_oracle::{DstOp, GrantState};
pub use super::dst_session::run_seed_ops;
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, bob_identity};
use super::schema::SharedProjectSchema;

pub const STEPS_PER_SEED: usize = 24;

#[derive(Clone, Copy, Debug, Default)]
pub struct DstCoverage {
    pub owner_writes: usize,
    pub grants_read_only: usize,
    pub grants_read_watch: usize,
    pub revokes: usize,
    pub remote_reads_delivered: usize,
    pub remote_reads_capability_denied: usize,
    pub remote_reads_invalid_sig: usize,
    pub watch_opens_ok: usize,
    pub watch_opens_read_only_denied: usize,
    pub watch_polls_delivered: usize,
    pub watch_polls_pending: usize,
    pub watch_polls_revoked_denied: usize,
    pub watch_polls_not_found: usize,
    pub home_reopens: usize,
    pub home_closed_refusals: usize,
}

impl DstCoverage {
    pub fn add(&mut self, other: Self) {
        self.owner_writes += other.owner_writes;
        self.grants_read_only += other.grants_read_only;
        self.grants_read_watch += other.grants_read_watch;
        self.revokes += other.revokes;
        self.remote_reads_delivered += other.remote_reads_delivered;
        self.remote_reads_capability_denied += other.remote_reads_capability_denied;
        self.remote_reads_invalid_sig += other.remote_reads_invalid_sig;
        self.watch_opens_ok += other.watch_opens_ok;
        self.watch_opens_read_only_denied += other.watch_opens_read_only_denied;
        self.watch_polls_delivered += other.watch_polls_delivered;
        self.watch_polls_pending += other.watch_polls_pending;
        self.watch_polls_revoked_denied += other.watch_polls_revoked_denied;
        self.watch_polls_not_found += other.watch_polls_not_found;
        self.home_reopens += other.home_reopens;
        self.home_closed_refusals += other.home_closed_refusals;
    }

    pub fn record(&mut self, op: DstOp, outcome: &str, grant_before: GrantState) {
        match op {
            DstOp::OwnerWrite { .. } => self.owner_writes += 1,
            DstOp::GrantScopeAccess {
                access: ScopeAccess::ReadOnly,
                ..
            } => self.grants_read_only += 1,
            DstOp::GrantScopeAccess {
                access: ScopeAccess::ReadAndWatch,
                ..
            } => self.grants_read_watch += 1,
            DstOp::RevokeScopeAccess { .. } => self.revokes += 1,
            DstOp::ReopenHome => self.home_reopens += 1,
            DstOp::RemoteRead { .. } => match outcome {
                "Delivered" => self.remote_reads_delivered += 1,
                "AuthzCapabilityDenied" => self.remote_reads_capability_denied += 1,
                "AuthInvalidSignature" => self.remote_reads_invalid_sig += 1,
                "HomeClosed" => self.home_closed_refusals += 1,
                _ => {}
            },
            DstOp::RemoteWatchOpen { .. } => match outcome {
                "Ok" => self.watch_opens_ok += 1,
                "AuthzCapabilityDenied" if grant_before == GrantState::ActiveReadOnly => {
                    self.watch_opens_read_only_denied += 1;
                }
                "HomeClosed" => self.home_closed_refusals += 1,
                _ => {}
            },
            DstOp::RemoteWatchPoll { .. } => match outcome {
                "Delivered" => self.watch_polls_delivered += 1,
                "Pending" => self.watch_polls_pending += 1,
                "AuthzWatchCapabilityRevoked" => self.watch_polls_revoked_denied += 1,
                "WatchNotFound" => self.watch_polls_not_found += 1,
                "HomeClosed" => self.home_closed_refusals += 1,
                _ => {}
            },
            DstOp::CloseHome | DstOp::TamperAuthToken { .. } | DstOp::RestoreAuthToken { .. } => {}
        }
    }
}

pub fn make_auth_config() -> RemoteAuthConfig {
    let key = RemoteAuthKey::new("k1", [0x42_u8; 32]).unwrap();
    RemoteAuthConfig::new(
        WorthQueryAuthenticationAudience::new("daymark.remote").unwrap(),
        WorthQueryAuthenticationMethod::new("hmac-sha256").unwrap(),
        key,
    )
}

pub fn issue_principal_token(
    config: &RemoteAuthConfig,
    home: &WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
    principal_idx: usize,
    nonce_seed: u64,
) -> RemoteAuthToken {
    let principal = if principal_idx == 0 {
        alice_identity()
    } else {
        bob_identity()
    };
    let mut nonce = [0_u8; 16];
    nonce[..8].copy_from_slice(&nonce_seed.to_be_bytes());
    nonce[8..16].copy_from_slice(&(principal_idx as u64).to_be_bytes());
    config
        .issue_token(RemoteTokenIssueRequest {
            key_id: "k1".to_owned(),
            home_identity: home.home_identity().expect("directory home identity"),
            pinned_incarnation: None,
            principal,
            ttl: Duration::from_secs(3600),
            nonce,
        })
        .unwrap()
}

pub fn tamper_token(valid: &RemoteAuthToken) -> RemoteAuthToken {
    let mut bytes = valid.to_bytes();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    RemoteAuthToken::from_bytes(&bytes).unwrap()
}

/// Strictly 1-to-1 exact denial reason mapping — NEVER collapses distinct reasons!
pub fn classify_exact_denial(denial: &SharedAccessDenial) -> Result<&'static str, String> {
    use AuthenticationRefusalReason as Auth;
    use AuthorizationRefusalReason as Authz;
    use BudgetRefusalReason as Budget;
    use ProtocolRefusalReason as Proto;
    use WatchRefusalReason as Watch;
    match denial {
        SharedAccessDenial::AuthenticationRefused(Auth::MalformedToken) => Ok("AuthMalformedToken"),
        SharedAccessDenial::AuthenticationRefused(Auth::UnknownKey) => Ok("AuthUnknownKey"),
        SharedAccessDenial::AuthenticationRefused(Auth::InvalidSignature) => {
            Ok("AuthInvalidSignature")
        }
        SharedAccessDenial::AuthenticationRefused(Auth::HomeIdentityMismatch) => {
            Ok("AuthHomeIdentityMismatch")
        }
        SharedAccessDenial::AuthenticationRefused(Auth::StaleIncarnation) => {
            Ok("AuthStaleIncarnation")
        }
        SharedAccessDenial::AuthenticationRefused(Auth::IncarnationPinRequired) => {
            Ok("AuthIncarnationPinRequired")
        }
        SharedAccessDenial::AuthenticationRefused(Auth::TokenExpired) => Ok("AuthTokenExpired"),
        SharedAccessDenial::AuthenticationRefused(Auth::AdapterDenied) => Ok("AuthAdapterDenied"),
        SharedAccessDenial::AuthorizationRefused(Authz::CapabilityDenied) => {
            Ok("AuthzCapabilityDenied")
        }
        SharedAccessDenial::AuthorizationRefused(Authz::PrincipalResolutionDenied) => {
            Ok("AuthzPrincipalResolutionDenied")
        }
        SharedAccessDenial::AuthorizationRefused(Authz::ScopeNotFound) => Ok("AuthzScopeNotFound"),
        SharedAccessDenial::AuthorizationRefused(Authz::WatchCapabilityRevoked) => {
            Ok("AuthzWatchCapabilityRevoked")
        }
        SharedAccessDenial::AuthorizationRefused(Authz::StalePrincipal) => {
            Ok("AuthzStalePrincipal")
        }
        SharedAccessDenial::AuthorizationRefused(Authz::StaleScope) => Ok("AuthzStaleScope"),
        SharedAccessDenial::WatchRefused(Watch::WatchNotFound) => Ok("WatchNotFound"),
        SharedAccessDenial::WatchRefused(Watch::WatchClosed) => Ok("WatchClosed"),
        SharedAccessDenial::WatchRefused(Watch::WatchOverflow) => Ok("WatchOverflow"),
        SharedAccessDenial::ProtocolRefused(Proto::FrameTooLarge { .. }) => {
            Ok("ProtoFrameTooLarge")
        }
        SharedAccessDenial::BudgetRefused(Budget::WatchCapacityExhausted { .. }) => {
            Ok("BudgetWatchCapacityExhausted")
        }
        SharedAccessDenial::BudgetRefused(Budget::ResponseTooLarge { .. }) => {
            Ok("BudgetResponseTooLarge")
        }
        other => Err(format!("unexpected denial variant: {other:?}")),
    }
}

fn configured_seeds() -> Vec<u64> {
    let parse = |name: &str| std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok());
    match parse("WORTH_DST_SEED") {
        Some(seed) => vec![seed],
        None => (0..parse("WORTH_DST_SEEDS").unwrap_or(64)).collect(),
    }
}

#[test]
fn seeded_shared_access_dst_agrees_with_tla_oracle() {
    let seeds = configured_seeds();
    let trace_dir = std::env::var_os("WORTH_DST_TRACE_DIR").map(PathBuf::from);
    let workers = std::thread::available_parallelism()
        .map_or(4, usize::from)
        .min(8)
        .min(seeds.len().max(1));
    let next = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let total = Mutex::new(DstCoverage::default());

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while let Some(&seed) = seeds.get(next.fetch_add(1, Ordering::Relaxed)) {
                    match run_seed_ops(seed, None) {
                        Ok(out) => {
                            if let Some(dir) = trace_dir.as_deref() {
                                let _ = write_trace(dir, seed, &out.trace_ndjson);
                            }
                            total.lock().unwrap().add(out.coverage);
                        }
                        Err(err) => failures.lock().unwrap().push(err),
                    }
                }
            });
        }
    });

    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} shared-access DST seeds failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    let total = total.into_inner().unwrap();
    eprintln!(
        "shared-access DST: {} seeds x {STEPS_PER_SEED} steps, {total:?}",
        seeds.len()
    );
    if seeds.len() >= 64 {
        assert_full_coverage(&total);
    }
}

fn assert_full_coverage(total: &DstCoverage) {
    for (name, count) in [
        ("owner writes", total.owner_writes),
        ("read-only grants", total.grants_read_only),
        ("read+watch grants", total.grants_read_watch),
        ("revokes", total.revokes),
        ("remote reads delivered", total.remote_reads_delivered),
        (
            "remote reads capability denied",
            total.remote_reads_capability_denied,
        ),
        (
            "remote reads invalid signature",
            total.remote_reads_invalid_sig,
        ),
        ("watch opens ok", total.watch_opens_ok),
        (
            "read-only watch open denied",
            total.watch_opens_read_only_denied,
        ),
        ("watch polls delivered", total.watch_polls_delivered),
        ("watch polls pending", total.watch_polls_pending),
        (
            "watch polls revoked denied",
            total.watch_polls_revoked_denied,
        ),
        ("watch polls not found", total.watch_polls_not_found),
        ("home reopens", total.home_reopens),
        ("home closed refusals", total.home_closed_refusals),
    ] {
        assert!(count > 0, "64-seed DST never exercised {name}");
    }
}

fn write_trace(dir: &Path, seed: u64, ndjson: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(format!("seed-{seed}.ndjson")), ndjson)
}
