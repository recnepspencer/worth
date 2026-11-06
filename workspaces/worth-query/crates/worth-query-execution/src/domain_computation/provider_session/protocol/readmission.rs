//! Portable session grant issuance and the separate live preparation receiver.
use super::{
    WorthQueryAdmittedProviderExecutionPlan, WorthQueryProviderExecutionPlanContract,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionTokenAdmission,
};

mod prepared;
pub use prepared::{
    WorthQueryPreparedProviderSession, WorthQuerySessionBoundReadsAndEffects,
    WorthQuerySessionEffectAuthority, WorthQuerySessionPrepareOutcome,
    WorthQuerySessionReadAuthority,
};

/// Identity and authority only. No run, request, provider or product lease.
/// Physical readmission begins when this grant reaches its live receiver.
#[derive(Debug)]
pub struct WorthQueryProviderSessionGrant {
    contract: WorthQueryProviderExecutionPlanContract,
    admission: WorthQueryProviderSessionTokenAdmission,
    counters: WorthQueryProviderSessionProtocolCounters,
}

/// Scoped custody of the actual run, product and provider used for preparation.
/// No physical session exists until the grant is checked against this receiver.
#[derive(Debug)]
pub struct WorthQueryProviderSessionReceiver<'run> {
    plan: WorthQueryAdmittedProviderExecutionPlan<'run>,
}

pub(super) fn issue(
    plan: WorthQueryAdmittedProviderExecutionPlan<'_>,
    admission: WorthQueryProviderSessionTokenAdmission,
) -> (
    WorthQueryProviderSessionGrant,
    WorthQueryProviderSessionReceiver<'_>,
) {
    let grant = WorthQueryProviderSessionGrant {
        contract: plan.contract.clone(),
        admission,
        counters: plan.counters,
    };
    (grant, WorthQueryProviderSessionReceiver { plan })
}

impl WorthQueryProviderSessionGrant {
    pub fn plan_identity(&self) -> &str {
        self.contract.identity()
    }
    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }
}
