//! Immutable resource decisions, prepared once before invocation admission.

use std::sync::Arc;

use super::*;

#[cfg(test)]
mod tests;
use crate::domain_computation::execution_resource_admission::admission_plan_digest::admitted_plan_identity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedExecutionResourcePlan {
    pub(super) basis: Arc<PreparedExecutionResourceBasis>,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct PreparedExecutionResourceBasis {
    pub(super) contract_identity: Arc<str>,
    pub(super) request: Arc<WorthQueryExecutionResourceRequest>,
    pub(super) request_identity: Arc<str>,
    pub(super) envelope_identity: Arc<str>,
    pub(super) envelope: Arc<WorthQueryExecutionResourceEnvelope>,
    pub(super) support_snapshot: WorthQueryExecutionResourceSupportSnapshot,
    pub(super) strategy: WorthQueryExecutionStrategyContract,
    pub(super) posture: WorthQueryExecutionResourceAdmissionPosture,
}

impl PreparedExecutionResourcePlan {
    pub(crate) fn new(
        contract_identity: String,
        request: &WorthQueryExecutionResourceRequest,
        support_snapshot: WorthQueryExecutionResourceSupportSnapshot,
        strategy: WorthQueryExecutionStrategyContract,
    ) -> Self {
        let request_identity = request.canonical_identity().into();
        let envelope_identity = admitted_envelope_identity(strategy.envelope()).into();
        let envelope = Arc::new(strategy.envelope().clone());
        let posture = if strategy.envelope().degradation().is_some() {
            WorthQueryExecutionResourceAdmissionPosture::Degraded
        } else {
            WorthQueryExecutionResourceAdmissionPosture::Exact
        };
        Self {
            basis: Arc::new(PreparedExecutionResourceBasis {
                contract_identity: contract_identity.into(),
                request: Arc::new(request.clone()),
                request_identity,
                envelope_identity,
                envelope,
                support_snapshot,
                strategy,
                posture,
            }),
        }
    }

    pub(crate) fn support(&self) -> &WorthQueryExecutionResourceSupportSnapshot {
        &self.basis.support_snapshot
    }

    pub(crate) fn bind(
        &self,
        binding_identity: &str,
        counters: WorthQueryExecutionResourceAdmissionCounters,
    ) -> WorthQueryAdmittedExecutionResourcePlan {
        WorthQueryAdmittedExecutionResourcePlan {
            identity: admitted_plan_identity(
                binding_identity,
                &self.basis.contract_identity,
                &self.basis.request_identity,
                self.basis.support_snapshot.identity(),
                self.basis.strategy.name().as_str(),
                &self.basis.envelope_identity,
            )
            .into(),
            binding_identity: binding_identity.into(),
            basis: Arc::clone(&self.basis),
            counters,
        }
    }

    pub(crate) fn bind_admitted<Stop>(
        &self,
        binding_identity: &str,
        counters: WorthQueryExecutionResourceAdmissionCounters,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<WorthQueryAdmittedExecutionResourcePlan, PreparedResourcePlanAdmissionStop<Stop>>
    {
        use PreparedResourcePlanAdmissionStop as Refusal;
        // Read the six selected text headers and measure all seven framed
        // parts only after their entry and measurement-walk claims.
        admit(6 + 7, 0).map_err(Refusal::Admission)?;
        let parts = [
            ("", "worth_query_admitted_execution_resource_plan_v1"),
            ("binding:", binding_identity),
            ("contract:", self.basis.contract_identity.as_ref()),
            ("request:", self.basis.request_identity.as_ref()),
            ("support:", self.basis.support_snapshot.identity()),
            ("strategy:", self.basis.strategy.name().as_str()),
            ("envelope:", self.basis.envelope_identity.as_ref()),
        ];
        let input = parts
            .iter()
            .try_fold(0_u64, |sum, (prefix, value)| {
                sum.checked_add(8)?
                    .checked_add(u64::try_from(prefix.len()).ok()?)?
                    .checked_add(u64::try_from(value.len()).ok()?)
            })
            .ok_or(Refusal::AccountingOverflow)?;
        let blocks = input
            .checked_add(72)
            .map(|bytes| bytes / 64)
            .ok_or(Refusal::AccountingOverflow)?;
        let binding_bytes =
            u64::try_from(binding_identity.len()).map_err(|_| Refusal::AccountingOverflow)?;
        // One rendered String, two Arc<str> allocations including their
        // reference-count headers, and one shallow immutable-basis owner.
        let arc_header = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>();
        let arc_backing = |length| {
            let payload = std::alloc::Layout::array::<u8>(length).ok()?;
            let (layout, _) = arc_header.extend(payload).ok()?;
            u64::try_from(layout.pad_to_align().size()).ok()
        };
        let bytes = arc_backing(binding_identity.len())
            .and_then(|bytes| bytes.checked_add(arc_backing(64)?))
            .and_then(|bytes| bytes.checked_add(64))
            .ok_or(Refusal::AccountingOverflow)?;
        let work = input
            .checked_add(blocks)
            .and_then(|work| work.checked_add(32 + 64 + 64 + 1))
            .and_then(|work| work.checked_add((2 * arc_header.size()) as u64))
            .and_then(|work| work.checked_add(binding_bytes))
            .and_then(|work| {
                work.checked_add(
                    std::mem::size_of::<WorthQueryAdmittedExecutionResourcePlan>() as u64,
                )
            })
            .ok_or(Refusal::AccountingOverflow)?;
        admit(work, bytes).map_err(Refusal::Admission)?;
        Ok(self.bind(binding_identity, counters))
    }
}

#[derive(Debug)]
pub(crate) enum PreparedResourcePlanAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}
