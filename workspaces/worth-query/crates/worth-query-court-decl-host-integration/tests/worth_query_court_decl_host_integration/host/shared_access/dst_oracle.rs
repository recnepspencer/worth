//! Reference oracle transcribed directly from `specs/tla/shared_access/SharedAccess.tla`
//! and strictly passive seed+observation scheduler (ZERO oracle feedback).

use worth_query_host::facade::shared_access::ScopeAccess;

pub const PRINCIPALS: [&str; 2] = ["Alice", "Bob"];
pub const SCOPES: [&str; 2] = ["Project1", "Project2"];

pub type DeliveredRow = (String, u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrantState {
    None,
    ActiveReadOnly,
    ActiveReadAndWatch,
    Revoked,
}

impl GrantState {
    pub const fn as_tla(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::ActiveReadOnly => "ActiveReadOnly",
            Self::ActiveReadAndWatch => "ActiveReadAndWatch",
            Self::Revoked => "Revoked",
        }
    }

    pub const fn has_read(self) -> bool {
        matches!(self, Self::ActiveReadOnly | Self::ActiveReadAndWatch)
    }

    pub const fn has_watch(self) -> bool {
        matches!(self, Self::ActiveReadAndWatch)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DstOp {
    OwnerWrite {
        scope: usize,
    },
    GrantScopeAccess {
        principal: usize,
        scope: usize,
        access: ScopeAccess,
    },
    RevokeScopeAccess {
        principal: usize,
        scope: usize,
    },
    RemoteRead {
        principal: usize,
        scope: usize,
    },
    RemoteWatchOpen {
        principal: usize,
        scope: usize,
    },
    RemoteWatchPoll {
        principal: usize,
        scope: usize,
    },
    CloseHome,
    ReopenHome,
    TamperAuthToken {
        principal: usize,
    },
    RestoreAuthToken {
        principal: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedStep {
    pub outcome: &'static str,
    pub rows: Vec<DeliveredRow>,
}

impl ExpectedStep {
    fn ok() -> Self {
        Self {
            outcome: "Ok",
            rows: Vec::new(),
        }
    }

    fn ok_row(scope: usize, version: u64) -> Self {
        Self {
            outcome: "Ok",
            rows: vec![(SCOPES[scope].to_owned(), version)],
        }
    }

    fn delivered_row(scope: usize, version: u64) -> Self {
        Self {
            outcome: "Delivered",
            rows: vec![(SCOPES[scope].to_owned(), version)],
        }
    }

    fn refused(outcome: &'static str) -> Self {
        Self {
            outcome,
            rows: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SharedAccessOracle {
    pub home_open: bool,
    pub incarnation: u64,
    pub scope_value: [u64; 2],
    pub durable_grants: [[GrantState; 2]; 2],
    pub auth_valid: [bool; 2],
    pub watches_open: [[bool; 2]; 2],
    pub watch_seen: [[u64; 2]; 2],
}

impl Default for SharedAccessOracle {
    fn default() -> Self {
        Self {
            home_open: true,
            incarnation: 1,
            scope_value: [0, 0],
            durable_grants: [[GrantState::None; 2]; 2],
            auth_valid: [true, true],
            watches_open: [[false; 2]; 2],
            watch_seen: [[0; 2]; 2],
        }
    }
}

impl SharedAccessOracle {
    pub fn apply(&mut self, op: DstOp) -> ExpectedStep {
        match op {
            DstOp::OwnerWrite { scope } => {
                self.scope_value[scope] += 1;
                ExpectedStep::ok_row(scope, self.scope_value[scope])
            }
            DstOp::GrantScopeAccess {
                principal,
                scope,
                access,
            } => {
                self.durable_grants[principal][scope] = match access {
                    ScopeAccess::ReadOnly => GrantState::ActiveReadOnly,
                    ScopeAccess::ReadAndWatch => GrantState::ActiveReadAndWatch,
                };
                ExpectedStep::ok()
            }
            DstOp::RevokeScopeAccess { principal, scope } => {
                self.durable_grants[principal][scope] = GrantState::Revoked;
                ExpectedStep::ok()
            }
            DstOp::RemoteRead { principal, scope } => self.apply_remote_read(principal, scope),
            DstOp::RemoteWatchOpen { principal, scope } => {
                self.apply_remote_watch_open(principal, scope)
            }
            DstOp::RemoteWatchPoll { principal, scope } => {
                self.apply_remote_watch_poll(principal, scope)
            }
            DstOp::CloseHome => {
                self.home_open = false;
                self.watches_open = [[false; 2]; 2];
                self.watch_seen = [[0; 2]; 2];
                ExpectedStep::ok()
            }
            DstOp::ReopenHome => {
                self.home_open = true;
                self.incarnation += 1;
                self.watches_open = [[false; 2]; 2];
                self.watch_seen = [[0; 2]; 2];
                ExpectedStep::ok()
            }
            DstOp::TamperAuthToken { principal } => {
                self.auth_valid[principal] = false;
                ExpectedStep::ok()
            }
            DstOp::RestoreAuthToken { principal } => {
                self.auth_valid[principal] = true;
                ExpectedStep::ok()
            }
        }
    }

    fn apply_remote_read(&self, p: usize, s: usize) -> ExpectedStep {
        if !self.home_open {
            return ExpectedStep::refused("HomeClosed");
        }
        if !self.auth_valid[p] {
            return ExpectedStep::refused("AuthInvalidSignature");
        }
        if !self.durable_grants[p][s].has_read() {
            return ExpectedStep::refused("AuthzCapabilityDenied");
        }
        ExpectedStep::delivered_row(s, self.scope_value[s])
    }

    fn apply_remote_watch_open(&mut self, p: usize, s: usize) -> ExpectedStep {
        if !self.home_open {
            return ExpectedStep::refused("HomeClosed");
        }
        if !self.auth_valid[p] {
            return ExpectedStep::refused("AuthInvalidSignature");
        }
        if !self.durable_grants[p][s].has_watch() {
            return ExpectedStep::refused("AuthzCapabilityDenied");
        }
        self.watches_open[p][s] = true;
        self.watch_seen[p][s] = self.scope_value[s];
        ExpectedStep::ok_row(s, self.scope_value[s])
    }

    fn apply_remote_watch_poll(&mut self, p: usize, s: usize) -> ExpectedStep {
        if !self.home_open {
            return ExpectedStep::refused("HomeClosed");
        }
        if !self.auth_valid[p] {
            return ExpectedStep::refused("AuthInvalidSignature");
        }
        if !self.watches_open[p][s] {
            return ExpectedStep::refused("WatchNotFound");
        }
        if !self.durable_grants[p][s].has_watch() {
            self.watches_open[p][s] = false;
            self.watch_seen[p][s] = 0;
            return ExpectedStep::refused("AuthzWatchCapabilityRevoked");
        }
        if self.scope_value[s] > self.watch_seen[p][s] {
            self.watch_seen[p][s] = self.scope_value[s];
            ExpectedStep::delivered_row(s, self.scope_value[s])
        } else {
            ExpectedStep::refused("Pending")
        }
    }
}

/// Passive post-state observed strictly from the real Directory Home and remote server.
#[derive(Clone, Debug)]
pub struct PassivePostState {
    pub home_open: bool,
    pub incarnation: u64,
    pub project_values: [u64; 2],
    pub grants: [[GrantState; 2]; 2],
    pub watches_open: [[bool; 2]; 2],
}

/// Strictly passive scheduling inputs observed from the real runtime (NEVER the oracle).
#[derive(Clone, Debug)]
pub struct PassiveScheduleInput {
    pub home_open: bool,
    pub incarnation: u64,
    pub project_values: [u64; 2],
    pub durable_grants: [[GrantState; 2]; 2],
    pub server_watches_open: [[bool; 2]; 2],
    pub watch_last_delivered: [[u64; 2]; 2],
    pub token_tampered: [bool; 2],
}

impl PassiveScheduleInput {
    pub fn initial(
        incarnation: u64,
        project_values: [u64; 2],
        grants: [[GrantState; 2]; 2],
    ) -> Self {
        Self {
            home_open: true,
            incarnation,
            project_values,
            durable_grants: grants,
            server_watches_open: [[false; 2]; 2],
            watch_last_delivered: [[0; 2]; 2],
            token_tampered: [false; 2],
        }
    }
}

/// Deterministic xorshift64 PRNG for seeded simulation walks.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x517c_c1b7_2722_0a95 | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn index(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
}

/// Picks the next action strictly from `rng` + passive runtime observations.
pub fn pick_next_op(rng: &mut Rng, observed: &PassiveScheduleInput, step: usize) -> DstOp {
    if !observed.home_open {
        return match rng.index(5) {
            0 => DstOp::RemoteRead {
                principal: rng.index(2),
                scope: rng.index(2),
            },
            1 => DstOp::RemoteWatchPoll {
                principal: rng.index(2),
                scope: rng.index(2),
            },
            _ => DstOp::ReopenHome,
        };
    }
    let mut candidates = Vec::with_capacity(64);
    for s in 0..2 {
        candidates.push(DstOp::OwnerWrite { scope: s });
    }
    for p in 0..2 {
        for s in 0..2 {
            push_pair_candidates(rng, observed, (p, s), &mut candidates);
        }
        candidates.push(if observed.token_tampered[p] {
            DstOp::RestoreAuthToken { principal: p }
        } else {
            DstOp::TamperAuthToken { principal: p }
        });
    }
    if step >= 10 && observed.incarnation < 2 {
        candidates.push(DstOp::CloseHome);
        candidates.push(DstOp::CloseHome);
    }
    candidates[rng.index(candidates.len())]
}

fn push_pair_candidates(
    rng: &mut Rng,
    observed: &PassiveScheduleInput,
    pair: (usize, usize),
    out: &mut Vec<DstOp>,
) {
    let (principal, scope) = pair;
    let grant = observed.durable_grants[principal][scope];
    let watch_open = observed.server_watches_open[principal][scope];
    out.push(DstOp::RemoteRead { principal, scope });
    out.push(DstOp::RemoteWatchOpen { principal, scope });
    out.push(DstOp::RemoteWatchPoll { principal, scope });
    if grant == GrantState::None {
        let access = if rng.index(3) == 0 {
            ScopeAccess::ReadOnly
        } else {
            ScopeAccess::ReadAndWatch
        };
        push_repeated(
            out,
            DstOp::GrantScopeAccess {
                principal,
                scope,
                access,
            },
            2,
        );
    }
    if grant.has_watch() && !watch_open {
        push_repeated(out, DstOp::RemoteWatchOpen { principal, scope }, 3);
    }
    if watch_open {
        if observed.project_values[scope] > observed.watch_last_delivered[principal][scope]
            || !grant.has_watch()
        {
            push_repeated(out, DstOp::RemoteWatchPoll { principal, scope }, 5);
        } else {
            push_repeated(out, DstOp::OwnerWrite { scope }, 2);
        }
    }
    if grant.has_read() {
        let count = if watch_open { 2 } else { 1 };
        push_repeated(out, DstOp::RevokeScopeAccess { principal, scope }, count);
    }
}

fn push_repeated(out: &mut Vec<DstOp>, op: DstOp, count: usize) {
    for _ in 0..count {
        out.push(op);
    }
}
