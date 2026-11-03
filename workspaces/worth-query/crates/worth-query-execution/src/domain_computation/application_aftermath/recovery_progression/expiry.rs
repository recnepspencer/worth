//! Runtime-clock recovery expiry evaluation (R8.31 / Gate 8.7).

use worth_foundational::facade::AspectValue;
use worth_proof::{
    evaluate_freshness, AuthorityRevalidationRequired, CurrentValidity, EvaluatedFreshness,
    FreshnessEvaluation, FreshnessPolicy, FreshnessScopedBasis, FreshnessVerdict,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::runtime_time::{
    WorthQueryRuntimeClock, WorthQueryRuntimeTimeSample,
};

use super::super::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleAuthorityIdentity,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};

#[derive(Debug)]
struct WorthQueryRecoveryExpiryBasis {
    expires_at_unix_ms: Option<u64>,
    handle: WorthQueryRecoveryHandleAuthorityIdentity,
}

struct WorthQueryRecoveryExpiryPolicy;

type WorthQueryRecoveryFreshnessEvaluation = FreshnessEvaluation<
    WorthQueryRuntimeClock,
    WorthQueryRecoveryExpiryPolicy,
    WorthQueryRecoveryExpiryBasis,
>;

/// Runtime-clock evidence that this exact handle remains current.
#[derive(Debug)]
pub struct WorthQueryRecoveryCurrentDecision {
    scoped: FreshnessScopedBasis<CurrentValidity, WorthQueryRecoveryFreshnessEvaluation>,
}

/// Runtime-clock evidence that this exact handle crossed its expiry deadline.
#[derive(Debug)]
pub struct WorthQueryRecoveryExpiryDecision {
    scoped:
        FreshnessScopedBasis<AuthorityRevalidationRequired, WorthQueryRecoveryFreshnessEvaluation>,
}

/// Typed result of sampling the runtime clock for one exact recovery handle.
#[derive(Debug)]
pub enum WorthQueryRecoveryExpiryEvaluation {
    Current(WorthQueryRecoveryCurrentDecision),
    Expired(WorthQueryRecoveryExpiryDecision),
}

impl FreshnessPolicy<WorthQueryRuntimeClock, WorthQueryRecoveryExpiryBasis>
    for WorthQueryRecoveryExpiryPolicy
{
    fn classify(
        &self,
        basis: &WorthQueryRecoveryExpiryBasis,
        observed_at: &WorthQueryRuntimeTimeSample,
    ) -> FreshnessVerdict {
        let AspectValue::UInt64(now_ms) = *observed_at.value() else {
            return FreshnessVerdict::AuthorityRevalidationRequired;
        };
        if basis
            .expires_at_unix_ms
            .is_some_and(|deadline| now_ms >= deadline)
        {
            FreshnessVerdict::AuthorityRevalidationRequired
        } else {
            FreshnessVerdict::Current
        }
    }
}

impl WorthQueryRecoveryExpiryDecision {
    pub fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.scoped.basis().observed_at().value()
    }

    pub(super) fn handle_authority(&self) -> WorthQueryRecoveryHandleAuthorityIdentity {
        self.scoped.basis().basis().handle
    }
}

impl WorthQueryRecoveryCurrentDecision {
    pub fn sample(&self) -> &WorthQueryRuntimeTimeSample {
        self.scoped.basis().observed_at().value()
    }
}

/// Evaluate expiry from the runtime clock. Callers cannot supply the sample.
pub(crate) fn evaluate_expiry(
    handle: &WorthQueryRecoveryHandle,
    clock: &WorthQueryRuntimeClock,
) -> Result<WorthQueryRecoveryExpiryEvaluation, WorthQueryRecoveryHandleDenial> {
    let evaluated = evaluate_freshness(
        WorthQueryRecoveryExpiryBasis {
            expires_at_unix_ms: handle.binding().expires_at_unix_ms(),
            handle: handle.authority_identity(),
        },
        clock,
        &WorthQueryRecoveryExpiryPolicy,
    )
    .map_err(|_| {
        WorthQueryRecoveryHandleDenial::new(WorthQueryRecoveryHandleDenialKind::Expired)
    })?;
    match evaluated {
        EvaluatedFreshness::Current(scoped) => Ok(WorthQueryRecoveryExpiryEvaluation::Current(
            WorthQueryRecoveryCurrentDecision { scoped },
        )),
        EvaluatedFreshness::AuthorityRevalidationRequired(scoped) => {
            Ok(WorthQueryRecoveryExpiryEvaluation::Expired(
                WorthQueryRecoveryExpiryDecision { scoped },
            ))
        }
        EvaluatedFreshness::StaleReadable(_) | EvaluatedFreshness::RebindRequired(_) => Err(
            WorthQueryRecoveryHandleDenial::new(WorthQueryRecoveryHandleDenialKind::Expired),
        ),
    }
}

pub(super) fn deny_if_expired(
    handle: &WorthQueryRecoveryHandle,
    clock: &WorthQueryRuntimeClock,
) -> Result<(), WorthQueryRecoveryHandleDenial> {
    if matches!(
        evaluate_expiry(handle, clock)?,
        WorthQueryRecoveryExpiryEvaluation::Expired(_)
    ) {
        Err(WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::Expired,
        ))
    } else {
        Ok(())
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Record an expiry decision from this runtime's clock.
    pub fn evaluate_recovery_expiry(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<WorthQueryRecoveryExpiryEvaluation, WorthQueryRecoveryHandleDenial> {
        handle.ensure_live()?;
        evaluate_expiry(handle, &self.authorization_clock)
    }
}
