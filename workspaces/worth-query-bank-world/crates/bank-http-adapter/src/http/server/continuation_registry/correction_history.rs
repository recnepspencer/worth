//! One latest native page per authenticated chooser, with bounded transport replay.
use super::*;
use crate::http::protocol::{
    BankHttpCorrectionAlternative, BankHttpCorrectionHistoryOutcome, BankHttpCorrectionSource,
    BankHttpDenial, BankHttpDenialKind,
};
use crate::http::server::{
    authenticated_owner::BankHttpAuthenticatedOwner, mutation_refusal::denial,
};
use worth_query_host::facade::primary_graph::WorthQueryCorrectionAlternativesPage;

pub(in crate::http::server) struct CorrectionHistoryRecord {
    owner: BankHttpAuthenticatedOwner,
    source: BankHttpCorrectionSource,
    pub(super) expires_at: Instant,
    page: Option<Arc<WorthQueryCorrectionAlternativesPage>>,
    pending: Option<String>,
    initial_request_id: String,
    initial_outcome: BankHttpCorrectionHistoryOutcome,
    last_request_id: String,
    last_outcome: BankHttpCorrectionHistoryOutcome,
}
pub(in crate::http::server) enum HistoryResumeAdmission {
    Execute(Arc<WorthQueryCorrectionAlternativesPage>),
    Replay(BankHttpCorrectionHistoryOutcome),
    Denied(BankHttpDenial),
}

impl BankHttpContinuationRegistry {
    pub(in crate::http::server) fn correction_initial_has_capacity(&mut self) -> bool {
        self.purge_expired();
        self.records.len() < self.capacity
    }

    pub(in crate::http::server) fn replay_correction_initial(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        source: &BankHttpCorrectionSource,
        request_id: &str,
    ) -> Option<(String, BankHttpCorrectionHistoryOutcome)> {
        self.purge_expired();
        let token = self
            .history_requests
            .get(&(owner.clone(), request_id.to_owned()))?;
        let ContinuationRecord::Correction(record) = self.records.get(token)? else {
            return None;
        };
        let outcome = if &record.source == source {
            record.initial_outcome.clone()
        } else {
            denied(request_id, denial(BankHttpDenialKind::Stale))
        };
        Some((token.clone(), outcome))
    }
    pub(in crate::http::server) fn register_correction_initial(
        &mut self,
        owner: BankHttpAuthenticatedOwner,
        source: BankHttpCorrectionSource,
        request_id: String,
        page: Arc<WorthQueryCorrectionAlternativesPage>,
    ) -> Result<BankHttpCorrectionHistoryOutcome, Arc<WorthQueryCorrectionAlternativesPage>> {
        self.purge_expired();
        if self.records.len() >= self.capacity {
            return Err(page);
        }
        let token = self.new_token();
        let outcome = delivered(&request_id, &token, &page);
        self.records.insert(
            token.clone(),
            ContinuationRecord::Correction(CorrectionHistoryRecord {
                owner: owner.clone(),
                source,
                expires_at: Instant::now() + self.lifetime,
                page: Some(page),
                pending: None,
                initial_request_id: request_id.clone(),
                initial_outcome: outcome.clone(),
                last_request_id: request_id.clone(),
                last_outcome: outcome.clone(),
            }),
        );
        self.history_requests.insert((owner, request_id), token);
        Ok(outcome)
    }
    pub(in crate::http::server) fn begin_correction_resume(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        source: &BankHttpCorrectionSource,
        request_id: &str,
        token: &str,
    ) -> HistoryResumeAdmission {
        self.purge_expired();
        let Some(ContinuationRecord::Correction(record)) = self.records.get_mut(token) else {
            return HistoryResumeAdmission::Denied(denial(BankHttpDenialKind::Stale));
        };
        if &record.owner != owner {
            return HistoryResumeAdmission::Denied(denial(BankHttpDenialKind::PermissionDenied));
        }
        if &record.source != source {
            return HistoryResumeAdmission::Denied(denial(BankHttpDenialKind::Stale));
        }
        if record.last_request_id == request_id {
            return HistoryResumeAdmission::Replay(record.last_outcome.clone());
        }
        if record.pending.is_some() {
            return HistoryResumeAdmission::Denied(denial(BankHttpDenialKind::Saturated));
        }
        if record
            .page
            .as_ref()
            .is_none_or(|page| page.continuation().is_none())
        {
            return HistoryResumeAdmission::Denied(denial(BankHttpDenialKind::Stale));
        }
        record.pending = Some(request_id.to_owned());
        HistoryResumeAdmission::Execute(record.page.take().expect("ready page"))
    }
    pub(in crate::http::server) fn complete_correction_resume(
        &mut self,
        token: &str,
        request_id: &str,
        page: Arc<WorthQueryCorrectionAlternativesPage>,
    ) -> Result<BankHttpCorrectionHistoryOutcome, Arc<WorthQueryCorrectionAlternativesPage>> {
        self.purge_expired();
        let Some(ContinuationRecord::Correction(record)) = self.records.get_mut(token) else {
            return Err(page);
        };
        if record.pending.as_deref() != Some(request_id) || record.page.is_some() {
            return Err(page);
        }
        let outcome = delivered(request_id, token, &page);
        record.page = Some(page);
        record.pending = None;
        record.last_request_id = request_id.to_owned();
        record.last_outcome = outcome.clone();
        record.expires_at = Instant::now() + self.lifetime;
        Ok(outcome)
    }
    pub(in crate::http::server) fn deny_correction_replay(
        &mut self,
        token: &str,
        cause: BankHttpDenial,
    ) {
        if let Some(ContinuationRecord::Correction(record)) = self.records.get_mut(token) {
            if let Some(page) = record.page.take() {
                self.retired_pages.push(page);
            }
            record.initial_outcome = denied(&record.initial_request_id, cause);
            record.last_outcome = denied(&record.last_request_id, cause);
            record.pending = None;
        }
    }
    pub(in crate::http::server) fn fail_correction_resume(
        &mut self,
        token: &str,
        request_id: &str,
        cause: BankHttpDenial,
        previous: Arc<WorthQueryCorrectionAlternativesPage>,
        retain_previous: bool,
    ) -> Option<Arc<WorthQueryCorrectionAlternativesPage>> {
        self.purge_expired();
        if let Some(ContinuationRecord::Correction(record)) = self.records.get_mut(token)
            && record.pending.as_deref() == Some(request_id)
            && record.page.is_none()
        {
            record.pending = None;
            record.last_request_id = request_id.to_owned();
            record.last_outcome = denied(request_id, cause);
            if retain_previous {
                record.page = Some(previous);
                return None;
            }
        }
        Some(previous)
    }
    /// Token possession lends only native page custody; fresh selection owns disclosure.
    pub(in crate::http::server) fn correction_page(
        &mut self,
        owner: &BankHttpAuthenticatedOwner,
        token: &str,
    ) -> Result<Arc<WorthQueryCorrectionAlternativesPage>, BankHttpDenial> {
        self.purge_expired();
        let Some(ContinuationRecord::Correction(record)) = self.records.get(token) else {
            return Err(denial(BankHttpDenialKind::Stale));
        };
        if &record.owner != owner {
            return Err(denial(BankHttpDenialKind::PermissionDenied));
        }
        record.page.clone().ok_or_else(|| {
            denial(if record.pending.is_some() {
                BankHttpDenialKind::Saturated
            } else {
                BankHttpDenialKind::Stale
            })
        })
    }
}
fn delivered(
    request_id: &str,
    token: &str,
    page: &WorthQueryCorrectionAlternativesPage,
) -> BankHttpCorrectionHistoryOutcome {
    BankHttpCorrectionHistoryOutcome::Delivered {
        request_id: request_id.to_owned(),
        continuation: token.to_owned(),
        complete: page.continuation().is_none(),
        alternatives: page
            .entries()
            .iter()
            .map(|entry| BankHttpCorrectionAlternative {
                source: entry.source().clone().into(),
            })
            .collect(),
    }
}
fn denied(request_id: &str, cause: BankHttpDenial) -> BankHttpCorrectionHistoryOutcome {
    BankHttpCorrectionHistoryOutcome::Denied {
        request_id: Some(request_id.to_owned()),
        denial: cause,
    }
}
