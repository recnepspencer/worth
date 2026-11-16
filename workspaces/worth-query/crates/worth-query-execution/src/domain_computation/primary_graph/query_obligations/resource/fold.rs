//! Lost process memory never discharges an original resource occurrence.
use super::super::{QueryObligationDenial as D, QueryObligationLedger};
use super::*;
impl QueryObligationLedger {
    pub fn resources(&self) -> impl Iterator<Item = &QueryResourceObligation> {
        self.resources.values()
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn validate_resource(
        &self,
        record: &QueryResourceRecord,
        position: u64,
    ) -> Result<(), D> {
        match record {
            QueryResourceRecord::Retain(row) => {
                if row.identity.home() != self.home()
                    || row.identity.acquire_position().get() != position
                {
                    return Err(D::ObligationIdentityMismatch);
                }
                if self.resources.contains_key(&row.identity) {
                    return Err(D::DuplicateObligation);
                }
                if row.handoff != QueryResourceHandoff::default()
                    || row.original.declaration.retention
                        != worth_runtime_bridge::facade::BridgeOwnedAsyncRetention::ProcessLocal
                    || row.original.declaration.managed_queue_capacity.is_none()
                {
                    return Err(D::ObligationSubjectMismatch);
                }
            }
            QueryResourceRecord::Handoff { identity, handoff } => {
                let row = self.resources.get(identity).ok_or(D::UnknownObligation)?;
                if row.handoff.release.is_some() && &row.handoff != handoff {
                    return Err(D::ObligationSubjectMismatch);
                }
                if row.handoff.original_request.is_some()
                    && row.handoff.original_request != handoff.original_request
                {
                    return Err(D::ObligationSubjectMismatch);
                }
                if let Some(release) = &handoff.release {
                    match (&release.terminal, handoff.request) {
                        (Some(terminal), Some(request))
                            if terminal.handle() == request
                                && terminal.status()
                                    != worth_signal::facade::ResourceInFlightStatus::Active => {}
                        (None, None) if release.predecessor.is_none() => {}
                        _ => return Err(D::ObligationSubjectMismatch),
                    }
                }
            }
            QueryResourceRecord::Discharge { identity } => {
                let row = self.resources.get(identity).ok_or(D::UnknownObligation)?;
                if row.handoff.release.is_none() {
                    return Err(D::OwnedAsyncPending {
                        identity: *identity,
                    });
                }
            }
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::query_obligations) fn apply_resource(
        &mut self,
        record: QueryResourceRecord,
    ) {
        match record {
            QueryResourceRecord::Retain(row) => {
                self.resources.insert(row.identity, row);
            }
            QueryResourceRecord::Handoff { identity, handoff } => {
                self.resources
                    .get_mut(&identity)
                    .expect("validated resource")
                    .handoff = handoff
            }
            QueryResourceRecord::Discharge { identity } => {
                self.resources.remove(&identity);
            }
        }
    }
}
