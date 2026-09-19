//! Safe-retry transition — consumes proof of one completed re-dispatch (R8.66).

use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;

use super::super::external_effect::WorthQueryExternalEffectDispatch;
use super::super::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind,
};
use super::super::recovery_posture::WorthQueryDispatchOutboxDurabilityPosture;
use super::authority::WorthQueryRecoveryEffectAuthority;
use super::redispatch::WorthQueryPerformedExternalRedispatch;

/// Proof that safe-retry was admitted after a real re-dispatch and the handle
/// was consumed.
#[derive(Debug)]
pub struct WorthQueryRecoverySafeRetryAdmission {
    binding: WorthQueryRecoveryHandleBinding,
    dispatch: WorthQueryExternalEffectDispatch,
    outbox_durability: WorthQueryDispatchOutboxDurabilityPosture,
}

/// A denied retry returns the still-live handle to its custodian. A terminal
/// denial means the registry already ended the resource and cannot return it.
#[derive(Debug)]
pub enum WorthQueryRecoverySafeRetryDenial {
    Retained {
        denial: WorthQueryRecoveryHandleDenial,
        handle: Box<WorthQueryRecoveryHandle>,
    },
    Terminal(WorthQueryRecoveryHandleDenial),
}

impl WorthQueryRecoverySafeRetryDenial {
    pub const fn kind(&self) -> WorthQueryRecoveryHandleDenialKind {
        match self {
            Self::Retained { denial, .. } | Self::Terminal(denial) => denial.kind(),
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryRecoveryHandleDenial,
        Option<WorthQueryRecoveryHandle>,
    ) {
        match self {
            Self::Retained { denial, handle } => (denial, Some(*handle)),
            Self::Terminal(denial) => (denial, None),
        }
    }
}

impl WorthQueryRecoverySafeRetryAdmission {
    pub const fn binding(&self) -> &WorthQueryRecoveryHandleBinding {
        &self.binding
    }

    pub const fn dispatch(&self) -> &WorthQueryExternalEffectDispatch {
        &self.dispatch
    }

    /// Process-local outbox lifetime, stated rather than implied (R8.71).
    pub const fn outbox_durability(&self) -> WorthQueryDispatchOutboxDurabilityPosture {
        self.outbox_durability
    }
}

pub fn safe_retry_recovery_handle(
    handle: WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
    redispatch: WorthQueryPerformedExternalRedispatch,
) -> Result<WorthQueryRecoverySafeRetryAdmission, WorthQueryRecoverySafeRetryDenial> {
    let check = (|| {
        authority.ensure_performed_for(&handle)?;
        // Rules out swapping a redispatch proof performed for handle A into
        // safe-retry for handle B (same runtime, different binding). Rung 3:
        // both sides are runtime values, so this is a comparison rather than a
        // type. The substitution is performed — not merely asserted — by
        // `primary_graph::application_attempt::provider_execution::external_dispatch::
        // safe_retry_affinity_tests::
        // redispatch_performed_for_handle_a_cannot_safe_retry_handle_b`, and
        // the `None` arm by
        // `safe_retry_denies_when_the_handle_carries_no_co_committed_outbox`;
        // both fail if either arm is removed.
        if redispatch.handle_authority() != handle.authority_identity() {
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::CorrelationMismatch,
            ));
        }
        if !redispatch.dispatch().is_external_completion() {
            return Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture,
            ));
        }
        Ok(())
    })();
    if let Err(denial) = check {
        if denial.kind() == WorthQueryRecoveryHandleDenialKind::AlreadyTerminal {
            return Err(WorthQueryRecoverySafeRetryDenial::Terminal(denial));
        }
        return Err(WorthQueryRecoverySafeRetryDenial::Retained {
            denial,
            handle: Box::new(handle),
        });
    }
    let dispatch = redispatch.into_dispatch();
    Ok(WorthQueryRecoverySafeRetryAdmission {
        binding: handle
            .consume(WorthQueryRecoveryResourceTerminal::Completed)
            .map_err(WorthQueryRecoverySafeRetryDenial::Terminal)?,
        dispatch,
        outbox_durability: WorthQueryDispatchOutboxDurabilityPosture::StoreCapabilityRequired,
    })
}
