//! Open-incarnation loopback TCP session helper for Shared Access DST.

use std::time::Duration;

use worth_query_host::facade::{
    shared_access::{
        RemoteAuthToken, RemoteWatchId, RemoteWatchPollOutcome, ScopeAccess, ScopeGrantId,
        WorthQueryRemoteClient, WorthQueryRemoteServer, WorthQueryRemoteWatch,
    },
    WorthQueryApplicationHome,
};

use super::dst::classify_exact_denial;
use super::dst_oracle::{DstOp, GrantState, SCOPES};
use super::mutation_handlers::SharedProjectProgram;
use super::mutations::{alice_identity, bob_identity, ProjectId};
use super::program::owner_write_project;
use super::query::{ProjectQueryParams, ProjectStateRow};
use super::schema::SharedProjectSchema;

pub struct SeedState {
    pub valid_tokens: [RemoteAuthToken; 2],
    pub tampered_tokens: [RemoteAuthToken; 2],
    pub active_tokens: [RemoteAuthToken; 2],
    pub grant_ids: [[Option<ScopeGrantId>; 2]; 2],
    pub grant_states: [[GrantState; 2]; 2],
    pub project_values: [u64; 2],
    pub incarnation: u64,
}

impl SeedState {
    pub fn new(valid_tokens: [RemoteAuthToken; 2], tampered_tokens: [RemoteAuthToken; 2]) -> Self {
        Self {
            active_tokens: valid_tokens.clone(),
            valid_tokens,
            tampered_tokens,
            grant_ids: Default::default(),
            grant_states: [[GrantState::None; 2]; 2],
            project_values: [0, 0],
            incarnation: 1,
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
    pub fn observe_watches(&self) -> [[bool; 2]; 2] {
        [
            [self.watches[0][0].is_some(), self.watches[0][1].is_some()],
            [self.watches[1][0].is_some(), self.watches[1][1].is_some()],
        ]
    }

    pub fn execute(
        &mut self,
        state: &mut SeedState,
        op: DstOp,
    ) -> Result<(&'static str, u64), String> {
        match op {
            DstOp::OwnerWrite { scope } => {
                let next = state.project_values[scope] + 1;
                let committed = owner_write_project(self.home, SCOPES[scope], next);
                state.project_values[scope] = committed;
                Ok(("Ok", committed))
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
                state.grant_states[principal][scope] = GrantState::Revoked;
                Ok(("Ok", 0))
            }
            DstOp::RemoteRead { principal, scope } => self.execute_remote_read(principal, scope),
            DstOp::RemoteWatchOpen { principal, scope } => {
                self.execute_watch_open(state, principal, scope)
            }
            DstOp::RemoteWatchPoll { principal, scope } => {
                self.execute_watch_poll(state, principal, scope)
            }
            DstOp::TamperAuthToken { principal } => {
                state.active_tokens[principal] = state.tampered_tokens[principal].clone();
                self.clients[principal].set_auth_token(state.active_tokens[principal].clone());
                Ok(("Ok", 0))
            }
            DstOp::RestoreAuthToken { principal } => {
                state.active_tokens[principal] = state.valid_tokens[principal].clone();
                self.clients[principal].set_auth_token(state.active_tokens[principal].clone());
                Ok(("Ok", 0))
            }
            DstOp::CloseHome | DstOp::ReopenHome => unreachable!(),
        }
    }

    fn execute_grant(
        &mut self,
        state: &mut SeedState,
        args: (usize, usize, ScopeAccess),
    ) -> Result<(&'static str, u64), String> {
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
        state.grant_ids[principal][scope] = Some(grant.id().clone());
        state.grant_states[principal][scope] = match access {
            ScopeAccess::ReadOnly => GrantState::ActiveReadOnly,
            ScopeAccess::ReadAndWatch => GrantState::ActiveReadAndWatch,
        };
        Ok(("Ok", 0))
    }

    fn execute_remote_read(&mut self, p: usize, s: usize) -> Result<(&'static str, u64), String> {
        self.clients[p]
            .send_query::<SharedProjectSchema, SharedProjectProgram>(
                &ProjectId(SCOPES[s].to_owned()),
                &ProjectQueryParams,
            )
            .map_err(|e| format!("send_query failed: {e:?}"))?;
        pump_server_once(&mut self.server)?;
        match self.clients[p].recv_query::<SharedProjectSchema, SharedProjectProgram>() {
            Ok(rows) => {
                let row = rows
                    .first()
                    .ok_or_else(|| "remote read returned empty rows".to_owned())?;
                Ok(("Delivered", row.version))
            }
            Err(denial) => Ok((classify_exact_denial(&denial)?, 0)),
        }
    }

    fn execute_watch_open(
        &mut self,
        state: &SeedState,
        p: usize,
        s: usize,
    ) -> Result<(&'static str, u64), String> {
        self.clients[p]
            .send_watch_open::<SharedProjectSchema, SharedProjectProgram>(
                &ProjectId(SCOPES[s].to_owned()),
                &ProjectQueryParams,
            )
            .map_err(|e| format!("send_watch_open failed: {e:?}"))?;
        pump_server_once(&mut self.server)?;
        match self.clients[p].recv_watch_open::<SharedProjectSchema, SharedProjectProgram>() {
            Ok(mut watch) => {
                let init_ver = watch
                    .initial_rows()
                    .first()
                    .map(|r: &ProjectStateRow| r.version)
                    .unwrap_or(0);
                watch.set_auth_token(state.active_tokens[p].clone());
                let _ = self.watches[p][s].replace(watch);
                Ok(("Ok", init_ver))
            }
            Err(denial) => Ok((classify_exact_denial(&denial)?, 0)),
        }
    }

    fn execute_watch_poll(
        &mut self,
        state: &SeedState,
        p: usize,
        s: usize,
    ) -> Result<(&'static str, u64), String> {
        if let Some(watch) = self.watches[p][s].as_mut() {
            watch.set_auth_token(state.active_tokens[p].clone());
            watch.send_poll().map_err(|e| format!("send_poll: {e:?}"))?;
            pump_server_once(&mut self.server)?;
            match watch.recv_poll() {
                Ok(RemoteWatchPollOutcome::Delivered(rows)) => {
                    Ok(("Delivered", rows.first().map(|r| r.version).unwrap_or(0)))
                }
                Ok(RemoteWatchPollOutcome::Pending) => Ok(("Pending", 0)),
                Err(denial) => {
                    let label = classify_exact_denial(&denial)?;
                    if label == "AuthorizationRefused" || label == "WatchRefused" {
                        self.watches[p][s] = None;
                    }
                    Ok((label, 0))
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
            Ok((classify_exact_denial(&denial)?, 0))
        }
    }
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
