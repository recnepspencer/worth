use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use crate::{diagnostics::WorthServerCounters, WorthServerResolvedRequestContext};

use super::{
    clock::WorthServerProductSessionClock,
    counters::WorthServerProductSessionCounterSnapshot,
    lifecycle_gate::{WorthServerProductSessionDenial, WorthServerProductSessionDenialCode},
    SharedProductSessionTerminationObserver, WorthServerProductSession,
    WorthServerProductSessionCreationRequest, WorthServerProductSessionExpiryPosture,
    WorthServerProductSessionIdentity, WorthServerProductSessionLifecycle,
    WorthServerProductSessionTermination, WorthServerProductSessionTerminationKind,
};

#[derive(Clone, Debug)]
pub struct WorthServerProductSessionRegistry {
    instance: crate::WorthServerInstanceIdentity,
    records: Arc<Mutex<HashMap<String, WorthServerProductSession>>>,
    next_identity: Arc<AtomicU64>,
    counters: Arc<WorthServerCounters>,
    clock: Arc<dyn WorthServerProductSessionClock>,
    termination_observers: Arc<Vec<SharedProductSessionTerminationObserver>>,
}

impl WorthServerProductSessionRegistry {
    pub(crate) fn new(
        instance: crate::WorthServerInstanceIdentity,
        counters: Arc<WorthServerCounters>,
        clock: Arc<dyn WorthServerProductSessionClock>,
        termination_observers: Vec<SharedProductSessionTerminationObserver>,
    ) -> Self {
        Self {
            instance,
            records: Arc::new(Mutex::new(HashMap::new())),
            next_identity: Arc::new(AtomicU64::new(1)),
            counters,
            clock,
            termination_observers: Arc::new(termination_observers),
        }
    }

    pub fn instance_identity(&self) -> &crate::WorthServerInstanceIdentity {
        &self.instance
    }

    pub fn open_preview(
        &self,
        resolved_request_context: &WorthServerResolvedRequestContext,
        request: WorthServerProductSessionCreationRequest,
    ) -> WorthServerProductSession {
        self.open(
            resolved_request_context,
            request,
            WorthServerProductSessionLifecycle::ReadOnlyPreview,
        )
    }

    pub fn open_mutation(
        &self,
        resolved_request_context: &WorthServerResolvedRequestContext,
        request: WorthServerProductSessionCreationRequest,
    ) -> WorthServerProductSession {
        self.open(
            resolved_request_context,
            request,
            WorthServerProductSessionLifecycle::MutationDraft,
        )
    }

    pub fn lookup(
        &self,
        identity: &str,
        resolved_request_context: &WorthServerResolvedRequestContext,
    ) -> Option<WorthServerProductSession> {
        self.counters.increment_product_session_lookups_attempted();
        let mut records = self.records.lock().expect("product session registry lock");
        let session = records.get(identity).cloned()?;
        let was_expired = matches!(
            session.expiry_posture(),
            WorthServerProductSessionExpiryPosture::Expired { .. }
        );
        let refreshed = self.refresh_expiry(session);
        let newly_expired = matches!(
            refreshed.expiry_posture(),
            WorthServerProductSessionExpiryPosture::Expired { .. }
        ) && !was_expired;
        records.insert(identity.to_string(), refreshed.clone());
        drop(records);
        if newly_expired {
            self.notify_termination(
                refreshed.clone(),
                WorthServerProductSessionTerminationKind::Expired,
            );
        }
        let _ = resolved_request_context;
        Some(refreshed)
    }

    pub fn close(
        &self,
        identity: &str,
        resolved_request_context: &WorthServerResolvedRequestContext,
    ) -> Result<WorthServerProductSession, WorthServerProductSessionDenial> {
        self.counters.increment_product_session_lookups_attempted();
        let mut records = self.records.lock().expect("product session registry lock");
        let session = records.get(identity).cloned().ok_or_else(|| {
            self.counters
                .increment_product_session_lookup_denied_missing();
            WorthServerProductSessionDenial::new(
                WorthServerProductSessionDenialCode::UnknownProductSessionIdentity,
                format!("product session `{identity}` was not found"),
            )
        })?;
        if session.tenant_id()
            != resolved_request_context
                .request_context()
                .workspace_target()
                .tenant_id()
            || session.workspace_id()
                != resolved_request_context
                    .request_context()
                    .workspace_target()
                    .workspace_id()
        {
            self.counters
                .increment_product_session_lookup_denied_foreign();
            return Err(WorthServerProductSessionDenial::new(
                WorthServerProductSessionDenialCode::ForeignProductSession,
                format!("product session `{identity}` does not belong to this workspace"),
            ));
        }
        let refreshed = self.refresh_expiry(session);
        match refreshed.expiry_posture() {
            WorthServerProductSessionExpiryPosture::Expired { .. } => {
                let newly_expired = matches!(
                    records
                        .get(identity)
                        .map(WorthServerProductSession::expiry_posture),
                    Some(WorthServerProductSessionExpiryPosture::Active { .. })
                );
                records.insert(identity.to_string(), refreshed.clone());
                drop(records);
                self.counters
                    .increment_product_session_lookup_denied_expired();
                if newly_expired {
                    self.notify_termination(
                        refreshed,
                        WorthServerProductSessionTerminationKind::Expired,
                    );
                }
                return Err(WorthServerProductSessionDenial::new(
                    WorthServerProductSessionDenialCode::ExpiredProductSession,
                    format!("product session `{identity}` has expired"),
                ));
            }
            WorthServerProductSessionExpiryPosture::Closed { .. } => {
                drop(records);
                self.counters
                    .increment_product_session_lookup_denied_closed();
                return Err(WorthServerProductSessionDenial::new(
                    WorthServerProductSessionDenialCode::ClosedProductSession,
                    format!("product session `{identity}` is closed"),
                ));
            }
            WorthServerProductSessionExpiryPosture::Active { .. } => {}
        }
        let closed_at_epoch_millis = self.clock.current_time_millis();
        let closed = WorthServerProductSession::new(super::WorthServerProductSessionParts {
            server_instance: refreshed.server_instance().clone(),
            identity: refreshed.identity().clone(),
            lifecycle: WorthServerProductSessionLifecycle::Closed,
            expiry_posture: WorthServerProductSessionExpiryPosture::Closed {
                closed_at_epoch_millis,
            },
            operation_name: refreshed.operation_name().to_string(),
            tenant_id: refreshed.tenant_id().to_string(),
            workspace_id: refreshed.workspace_id().to_string(),
            branch_label: refreshed.branch_label().to_string(),
            basis_digest: refreshed.basis_digest().map(str::to_string),
        });
        records.insert(identity.to_string(), closed.clone());
        drop(records);
        self.counters.increment_product_session_closes_recorded();
        self.notify_termination(
            closed.clone(),
            WorthServerProductSessionTerminationKind::Closed,
        );
        Ok(closed)
    }

    pub fn counter_snapshot(&self) -> WorthServerProductSessionCounterSnapshot {
        let snapshot = self.counters.snapshot();
        WorthServerProductSessionCounterSnapshot {
            sessions_created: snapshot.product_sessions_created,
            preview_sessions_created: snapshot.product_session_preview_creations,
            mutation_sessions_created: snapshot.product_session_mutation_creations,
            lookups_attempted: snapshot.product_session_lookups_attempted,
            lookups_denied_missing: snapshot.product_session_lookups_denied_missing,
            lookups_denied_foreign: snapshot.product_session_lookups_denied_foreign,
            lookups_denied_expired: snapshot.product_session_lookups_denied_expired,
            lookups_denied_closed: snapshot.product_session_lookups_denied_closed,
            lookups_denied_moved: snapshot.product_session_lookups_denied_moved,
            lookups_denied_preview_for_mutation: snapshot
                .product_session_lookups_denied_preview_for_mutation,
            closes_recorded: snapshot.product_session_closes_recorded,
        }
    }

    pub(crate) fn record_denial(&self, code: WorthServerProductSessionDenialCode) {
        match code {
            WorthServerProductSessionDenialCode::CoordinationRequestDenied
            | WorthServerProductSessionDenialCode::CoordinationAdmissionDenied
            | WorthServerProductSessionDenialCode::CoordinationReadinessDenied => {}
            WorthServerProductSessionDenialCode::MissingProductSessionIdentity
            | WorthServerProductSessionDenialCode::UnknownProductSessionIdentity => self
                .counters
                .increment_product_session_lookup_denied_missing(),
            WorthServerProductSessionDenialCode::ForeignProductSession => self
                .counters
                .increment_product_session_lookup_denied_foreign(),
            WorthServerProductSessionDenialCode::ExpiredProductSession => self
                .counters
                .increment_product_session_lookup_denied_expired(),
            WorthServerProductSessionDenialCode::ClosedProductSession => self
                .counters
                .increment_product_session_lookup_denied_closed(),
            WorthServerProductSessionDenialCode::PreviewSessionCannotMutate => self
                .counters
                .increment_product_session_lookup_denied_preview_for_mutation(),
            WorthServerProductSessionDenialCode::SessionRebindRequired => self
                .counters
                .increment_product_session_lookup_denied_moved(),
        }
    }

    fn open(
        &self,
        resolved_request_context: &WorthServerResolvedRequestContext,
        request: WorthServerProductSessionCreationRequest,
        lifecycle: WorthServerProductSessionLifecycle,
    ) -> WorthServerProductSession {
        let created_at_epoch_millis = self.clock.current_time_millis();
        let identity = self.mint_identity(resolved_request_context);
        let session = WorthServerProductSession::new(super::WorthServerProductSessionParts {
            server_instance: self.instance.clone(),
            identity: identity.clone(),
            lifecycle,
            expiry_posture: WorthServerProductSessionExpiryPosture::Active {
                expires_at_epoch_millis: created_at_epoch_millis
                    .saturating_add(request.expiry_seconds().saturating_mul(1000)),
            },
            operation_name: request.operation_name().to_string(),
            tenant_id: resolved_request_context
                .request_context()
                .workspace_target()
                .tenant_id()
                .to_string(),
            workspace_id: resolved_request_context
                .request_context()
                .workspace_target()
                .workspace_id()
                .to_string(),
            branch_label: resolved_request_context
                .request_context()
                .branch_target()
                .canonical_label()
                .to_string(),
            basis_digest: request.basis_digest().map(str::to_string),
        });
        self.records
            .lock()
            .expect("product session registry lock")
            .insert(identity.as_str().to_string(), session.clone());
        self.counters.increment_product_sessions_created();
        match lifecycle {
            WorthServerProductSessionLifecycle::ReadOnlyPreview => {
                self.counters.increment_product_session_preview_creations()
            }
            WorthServerProductSessionLifecycle::MutationDraft => {
                self.counters.increment_product_session_mutation_creations()
            }
            WorthServerProductSessionLifecycle::Closed => {}
        }
        session
    }

    fn mint_identity(
        &self,
        resolved_request_context: &WorthServerResolvedRequestContext,
    ) -> WorthServerProductSessionIdentity {
        let sequence = self.next_identity.fetch_add(1, Ordering::Relaxed);
        let workspace_target = resolved_request_context
            .request_context()
            .workspace_target();
        let branch_target = resolved_request_context.request_context().branch_target();
        WorthServerProductSessionIdentity::new(format!(
            "product-session:{}:{}:{}:{}:{sequence}",
            self.instance.as_str(),
            workspace_target.tenant_id(),
            workspace_target.workspace_id(),
            branch_target.canonical_label(),
        ))
    }

    fn refresh_expiry(&self, session: WorthServerProductSession) -> WorthServerProductSession {
        let WorthServerProductSessionExpiryPosture::Active {
            expires_at_epoch_millis,
        } = session.expiry_posture()
        else {
            return session;
        };
        let observed_at_epoch_millis = self.clock.current_time_millis();
        if observed_at_epoch_millis < *expires_at_epoch_millis {
            return session;
        }
        WorthServerProductSession::new(super::WorthServerProductSessionParts {
            server_instance: session.server_instance().clone(),
            identity: session.identity().clone(),
            lifecycle: session.lifecycle(),
            expiry_posture: WorthServerProductSessionExpiryPosture::Expired {
                expires_at_epoch_millis: *expires_at_epoch_millis,
                observed_at_epoch_millis,
            },
            operation_name: session.operation_name().to_string(),
            tenant_id: session.tenant_id().to_string(),
            workspace_id: session.workspace_id().to_string(),
            branch_label: session.branch_label().to_string(),
            basis_digest: session.basis_digest().map(str::to_string),
        })
    }

    fn notify_termination(
        &self,
        session: WorthServerProductSession,
        kind: WorthServerProductSessionTerminationKind,
    ) {
        let termination = WorthServerProductSessionTermination::new(session, kind);
        for observer in self.termination_observers.iter() {
            observer.observe_termination(&termination);
        }
    }
}
