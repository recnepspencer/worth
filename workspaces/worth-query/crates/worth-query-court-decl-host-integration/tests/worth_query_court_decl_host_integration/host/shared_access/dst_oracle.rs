//! Reference oracle transcribed directly from `specs/tla/shared_access/SharedAccess.tla`
//! and passive NDJSON trace recorder for `tools/tla/run_tlc.py trace`.

use worth_query_host::facade::shared_access::ScopeAccess;

pub const PRINCIPALS: [&str; 2] = ["Alice", "Bob"];
pub const SCOPES: [&str; 2] = ["Project1", "Project2"];

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedStep {
    pub outcome: &'static str,
    pub value: u64,
}

impl ExpectedStep {
    const fn ok(value: u64) -> Self {
        Self {
            outcome: "Ok",
            value,
        }
    }
    const fn refused(outcome: &'static str) -> Self {
        Self { outcome, value: 0 }
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
                ExpectedStep::ok(self.scope_value[scope])
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
                ExpectedStep::ok(0)
            }
            DstOp::RevokeScopeAccess { principal, scope } => {
                self.durable_grants[principal][scope] = GrantState::Revoked;
                ExpectedStep::ok(0)
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
                ExpectedStep::ok(0)
            }
            DstOp::ReopenHome => {
                self.home_open = true;
                self.incarnation += 1;
                self.watches_open = [[false; 2]; 2];
                self.watch_seen = [[0; 2]; 2];
                ExpectedStep::ok(0)
            }
            DstOp::TamperAuthToken { principal } => {
                self.auth_valid[principal] = false;
                ExpectedStep::ok(0)
            }
            DstOp::RestoreAuthToken { principal } => {
                self.auth_valid[principal] = true;
                ExpectedStep::ok(0)
            }
        }
    }

    fn apply_remote_read(&self, p: usize, s: usize) -> ExpectedStep {
        if !self.home_open {
            return ExpectedStep::refused("HomeClosed");
        }
        if !self.auth_valid[p] {
            return ExpectedStep::refused("AuthenticationRefused");
        }
        if !self.durable_grants[p][s].has_read() {
            return ExpectedStep::refused("AuthorizationRefused");
        }
        ExpectedStep {
            outcome: "Delivered",
            value: self.scope_value[s],
        }
    }

    fn apply_remote_watch_open(&mut self, p: usize, s: usize) -> ExpectedStep {
        if !self.home_open {
            return ExpectedStep::refused("HomeClosed");
        }
        if !self.auth_valid[p] {
            return ExpectedStep::refused("AuthenticationRefused");
        }
        if !self.durable_grants[p][s].has_watch() {
            return ExpectedStep::refused("AuthorizationRefused");
        }
        self.watches_open[p][s] = true;
        self.watch_seen[p][s] = self.scope_value[s];
        ExpectedStep::ok(self.scope_value[s])
    }

    fn apply_remote_watch_poll(&mut self, p: usize, s: usize) -> ExpectedStep {
        if !self.home_open {
            return ExpectedStep::refused("HomeClosed");
        }
        if !self.auth_valid[p] {
            return ExpectedStep::refused("AuthenticationRefused");
        }
        if !self.watches_open[p][s] {
            return ExpectedStep::refused("WatchRefused");
        }
        if !self.durable_grants[p][s].has_watch() {
            self.watches_open[p][s] = false;
            self.watch_seen[p][s] = 0;
            return ExpectedStep::refused("AuthorizationRefused");
        }
        if self.scope_value[s] > self.watch_seen[p][s] {
            self.watch_seen[p][s] = self.scope_value[s];
            ExpectedStep {
                outcome: "Delivered",
                value: self.scope_value[s],
            }
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

pub fn pick_next_op(rng: &mut Rng, oracle: &SharedAccessOracle, step: usize) -> DstOp {
    if !oracle.home_open {
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
            push_pair_candidates(rng, oracle, (p, s), &mut candidates);
        }
        candidates.push(if oracle.auth_valid[p] {
            DstOp::TamperAuthToken { principal: p }
        } else {
            DstOp::RestoreAuthToken { principal: p }
        });
    }
    if step >= 10 && oracle.incarnation < 2 {
        candidates.push(DstOp::CloseHome);
        candidates.push(DstOp::CloseHome);
    }
    candidates[rng.index(candidates.len())]
}

fn push_pair_candidates(
    rng: &mut Rng,
    oracle: &SharedAccessOracle,
    pair: (usize, usize),
    out: &mut Vec<DstOp>,
) {
    let (principal, scope) = pair;
    let grant = oracle.durable_grants[principal][scope];
    let watch_open = oracle.watches_open[principal][scope];
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
        if oracle.scope_value[scope] > oracle.watch_seen[principal][scope] || !grant.has_watch() {
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
