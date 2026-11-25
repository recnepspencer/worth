//! Open-incarnation loopback TCP session helper with strictly server-side watch observation.

use std::time::Duration;

use worth_query_host::facade::{
    shared_access::{
        RemoteAuthToken, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess, ScopeGrantId,
        WorthQueryRemoteClient, WorthQueryRemoteServer, WorthQueryRemoteWatch,
    },
    WorthQueryApplicationHome,
};

use super::dst::classify_exact_denial;
use super::dst_oracle::{DeliveredRow, DstOp, SCOPES};
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, bob_identity, ProjectId};
use super::program::owner_write_project;
use super::query::{ProjectQueryParams, ProjectStateRow};
use super::schema::SharedProjectSchema;

pub struct SeedState {
    pub valid_tokens: [RemoteAuthToken; 2],
    pub tampered_tokens: [RemoteAuthToken; 2],
    pub active_tokens: [RemoteAuthToken; 2],
    pub token_tampered: [bool; 2],
    pub grant_ids: [[Option<ScopeGrantId>; 2]; 2],
    pub watch_last_delivered: [[u64; 2]; 2],
}

impl SeedState {
    pub fn new(valid_tokens: [RemoteAuthToken; 2], tampered_tokens: [RemoteAuthToken; 2]) -> Self {
        Self {
            active_tokens: valid_tokens.clone(),
            valid_tokens,
            tampered_tokens,
            token_tampered: [false; 2],
            grant_ids: Default::default(),
            watch_last_delivered: [[0; 2]; 2],
        }
    }
}

pub struct OpenSession<'home> {
    pub home: &'home WorthQueryApplicationHome<SharedProjectSchema, SharedProjectProgram>,
    pub server: WorthQueryRemoteServer<'home, SharedProjectSchema, SharedProjectProgram>,
    pub clients: [WorthQueryRemoteClient; 2],
    pub watches: [[Option<WorthQueryRemoteWatch<SharedProjectSchema, SharedProjectProgram>>; 2]; 2],
}

impl OpenSession<'_> {
    /// Passively observes real server-side watch ownership from `server.active_watches()`.
    pub fn observe_server_watches(&self) -> [[bool; 2]; 2] {
        let mut matrix = [[false; 2]; 2];
        let alice = alice_identity();
        let bob = bob_identity();
        for (_, principal, scope) in self.server.active_watches() {
            let p_idx = if principal == alice {
                0
            } else if principal == bob {
                1
            } else {
                continue;
            };
            let s_idx = if scope.0 == SCOPES[0] {
                0
            } else if scope.0 == SCOPES[1] {
                1
            } else {
                continue;
            };
            matrix[p_idx][s_idx] = true;
        }
        matrix
    }

    pub fn execute(
        &mut self,
        state: &mut SeedState,
        op: DstOp,
        current_values: [u64; 2],
    ) -> Result<(&'static str, Vec<DeliveredRow>), String> {
        match op {
            DstOp::OwnerWrite { scope } => {
                let next = current_values[scope] + 1;
                let committed = owner_write_project(self.home, SCOPES[scope], next);
                Ok(("Ok", vec![(SCOPES[scope].to_owned(), committed)]))
            }
            DstOp::GrantScopeAccess {
                principal,
                scope,
                access,
            } => self.execute_grant(state, (principal, scope, access)),
            DstOp::RevokeScopeAccess { principal, scope } => {
                let id = state.grant_ids[principal][scope]
                    .as_ref()
                    .ok_or_else(|| "revoke without grant id".to_owned())?;
                self.home
                    .revoke_scope_access(id)
                    .map_err(|e| format!("revoke_scope_access failed: {e:?}"))?;
                Ok(("Ok", Vec::new()))
            }
            DstOp::RemoteRead { principal, scope } => self.execute_remote_read(principal, scope),
            DstOp::RemoteWatchOpen { principal, scope } => {
                self.execute_watch_open(state, principal, scope)
            }
            DstOp::RemoteWatchPoll { principal, scope } => {
                self.execute_watch_poll(state, principal, scope)
            }
            DstOp::TamperAuthToken { principal } => {
                state.token_tampered[principal] = true;
                state.active_tokens[principal] = state.tampered_tokens[principal].clone();
                self.clients[principal].set_auth_token(state.active_tokens[principal].clone());
                Ok(("Ok", Vec::new()))
            }
            DstOp::RestoreAuthToken { principal } => {
                state.token_tampered[principal] = false;
                state.active_tokens[principal] = state.valid_tokens[principal].clone();
                self.clients[principal].set_auth_token(state.active_tokens[principal].clone());
                Ok(("Ok", Vec::new()))
            }
            DstOp::CloseHome | DstOp::ReopenHome => unreachable!(),
        }
    }

    fn execute_grant(
        &mut self,
        state: &mut SeedState,
        args: (usize, usize, ScopeAccess),
    ) -> Result<(&'static str, Vec<DeliveredRow>), String> {
        let (principal, scope, access) = args;
        let ext = if principal == 0 {
            alice_identity()
        } else {
            bob_identity()
        };
        let grant = self
            .home
            .grant_scope_access(ProjectId(SCOPES[scope].to_owned()), ext, access)
            .map_err(|e| format!("grant_scope_access failed: {e:?}"))?;
        if grant.committed_record_changes() == 0 {
            return Err("grant receipt reported zero committed record changes".to_owned());
        }
        state.grant_ids[principal][scope] = Some(grant.id().clone());
        Ok(("Ok", Vec::new()))
    }

    fn execute_remote_read(
        &mut self,
        p: usize,
        s: usize,
    ) -> Result<(&'static str, Vec<DeliveredRow>), String> {
        self.clients[p]
            .send_query::<SharedProjectSchema, SharedProjectProgram>(
                &ProjectId(SCOPES[s].to_owned()),
                &ProjectQueryParams,
            )
            .map_err(|e| format!("send_query failed: {e:?}"))?;
        pump_server_once(&mut self.server)?;
        match self.clients[p].recv_query::<SharedProjectSchema, SharedProjectProgram>() {
            Ok(rows) => Ok(("Delivered", collect_rows(&rows))),
            Err(denial) => Ok((classify_exact_denial(&denial)?, Vec::new())),
        }
    }

    fn execute_watch_open(
        &mut self,
        state: &mut SeedState,
        p: usize,
        s: usize,
    ) -> Result<(&'static str, Vec<DeliveredRow>), String> {
        self.clients[p]
            .send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
                &ProjectId(SCOPES[s].to_owned()),
                &ProjectQueryParams,
            )
            .map_err(|e| format!("send_watch_open failed: {e:?}"))?;
        pump_server_once(&mut self.server)?;
        match self.clients[p].recv_watch_open::<SharedProjectSchema, SharedProjectProgram>() {
            Ok(mut watch) => {
                let rows = collect_rows(watch.initial_rows());
                if let Some((_, ver)) = rows.first() {
                    state.watch_last_delivered[p][s] = *ver;
                }
                watch.set_auth_token(state.active_tokens[p].clone());
                if let Some(old_watch) = self.watches[p][s].replace(watch) {
                    drop(old_watch);
                    let _ = self.server.serve_pending();
                }
                Ok(("Ok", rows))
            }
            Err(denial) => Ok((classify_exact_denial(&denial)?, Vec::new())),
        }
    }

    fn execute_watch_poll(
        &mut self,
        state: &mut SeedState,
        p: usize,
        s: usize,
    ) -> Result<(&'static str, Vec<DeliveredRow>), String> {
        if let Some(watch) = self.watches[p][s].as_mut() {
            watch.set_auth_token(state.active_tokens[p].clone());
            watch.send_poll().map_err(|e| format!("send_poll: {e:?}"))?;
            pump_server_once(&mut self.server)?;
            match watch.recv_poll() {
                Ok(RemoteWatchPollOutcome::Delivered(rows)) => {
                    let collected = collect_rows(&rows);
                    if let Some((_, ver)) = collected.first() {
                        state.watch_last_delivered[p][s] = *ver;
                    }
                    Ok(("Delivered", collected))
                }
                Ok(RemoteWatchPollOutcome::Pending) => Ok(("Pending", Vec::new())),
                Err(denial) => {
                    let label = classify_exact_denial(&denial)?;
                    if label == "AuthzWatchCapabilityRevoked" || label == "WatchNotFound" {
                        self.watches[p][s] = None;
                        state.watch_last_delivered[p][s] = 0;
                    }
                    Ok((label, Vec::new()))
                }
            }
        } else {
            self.clients[p]
                .send_watch_poll_id(RemoteWatchId::new(u64::MAX))
                .map_err(|e| format!("send_watch_poll_id: {e:?}"))?;
            pump_server_once(&mut self.server)?;
            let Err(denial) =
                self.clients[p].recv_watch_poll_id::<SharedProjectSchema, SharedProjectProgram>()
            else {
                return Err("unknown watch id unexpectedly succeeded".to_owned());
            };
            Ok((classify_exact_denial(&denial)?, Vec::new()))
        }
    }
}

fn collect_rows(rows: &[ProjectStateRow]) -> Vec<DeliveredRow> {
    rows.iter()
        .map(|r| (r.project_key.clone(), r.version))
        .collect()
}

fn pump_server_once(
    server: &mut WorthQueryRemoteServer<'_, SharedProjectSchema, SharedProjectProgram>,
) -> Result<(), String> {
    if server
        .serve_once(Duration::from_secs(5))
        .map_err(|e| format!("serve_once: {e:?}"))?
    {
        Ok(())
    } else {
        Err("server.serve_once timed out waiting for loopback frame".to_owned())
    }
}
