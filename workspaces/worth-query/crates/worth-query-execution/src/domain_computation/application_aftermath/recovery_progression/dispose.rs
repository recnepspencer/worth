//! Dispose transition — terminal, consumes the handle (R8.29 / R8.30).

use crate::domain_computation::managed_run::WorthQueryRecoveryResourceTerminal;

use super::super::recovery_handle::{
    RelinquishOnDenial, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleBinding,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use super::authority::{require_fresh_effect_authority, WorthQueryRecoveryEffectAuthority};

/// Proof that dispose completed and the handle is gone.
#[derive(Debug)]
pub struct WorthQueryRecoveryDisposalReceipt {
    binding: WorthQueryRecoveryHandleBinding,
}

impl WorthQueryRecoveryDisposalReceipt {
    pub const fn binding(&self) -> &WorthQueryRecoveryHandleBinding {
        &self.binding
    }
}

/// Refusal preserves the exact live handle; a terminal denial owns no live resource.
/// ```compile_fail,E0599
/// fn cannot_copy(denial:worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalDenial) { let _=denial.clone(); }
/// ```
/// ```
/// fn transfer(denial:worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalDenial)->worth_query_execution::facade::primary_graph::WorthQueryRecoveryDisposalDenial {denial}
/// ```
#[derive(Debug)]
pub enum WorthQueryRecoveryDisposalDenial {
    Retained {
        denial: WorthQueryRecoveryHandleDenial,
        handle: Box<WorthQueryRecoveryHandle>,
    },
    Terminal(WorthQueryRecoveryHandleDenial),
}
impl WorthQueryRecoveryDisposalDenial {
    pub fn denial(&self) -> &WorthQueryRecoveryHandleDenial {
        match self {
            Self::Retained { denial, .. } | Self::Terminal(denial) => denial,
        }
    }
    pub fn kind(&self) -> WorthQueryRecoveryHandleDenialKind {
        self.denial().kind()
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

/// Disposes of a recovery handle, ending the custody it stands for.
///
/// `authority` must be fresh effect authority minted for this exact handle. On
/// success the handle is consumed as disposed and the receipt names its binding.
/// On refusal the actual handle is returned; only native terminal consumption returns none.
pub fn dispose_recovery_handle(
    handle: WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
) -> Result<WorthQueryRecoveryDisposalReceipt, WorthQueryRecoveryDisposalDenial> {
    if let Err(denial) = require_fresh_effect_authority(&handle, authority) {
        return Err(
            if denial.kind() == WorthQueryRecoveryHandleDenialKind::AlreadyTerminal {
                WorthQueryRecoveryDisposalDenial::Terminal(denial)
            } else {
                WorthQueryRecoveryDisposalDenial::Retained {
                    denial,
                    handle: Box::new(handle),
                }
            },
        );
    }
    handle
        .consume(WorthQueryRecoveryResourceTerminal::Disposed)
        .map(|binding| WorthQueryRecoveryDisposalReceipt { binding })
        .map_err(WorthQueryRecoveryDisposalDenial::Terminal)
}

/// Expire after `evaluate_expiry` proves the deadline. Records the sample via
/// the decision argument (R8.7 M3).
pub fn expire_recovery_handle(
    handle: WorthQueryRecoveryHandle,
    decision: &super::WorthQueryRecoveryExpiryDecision,
) -> Result<WorthQueryRecoveryHandleBinding, WorthQueryRecoveryHandleDenial> {
    let handle = handle.admit(|handle| {
        if decision.handle_authority() == handle.authority_identity() {
            Ok(())
        } else {
            Err(WorthQueryRecoveryHandleDenial::new(
                WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied,
            ))
        }
    })?;
    // `WorthQueryRecoveryExpiryDecision` is the expired branch of a typed
    // runtime-clock evaluation. Current evidence has a different type and
    // cannot reach this signature (R8.7 M2 / R8.31).
    handle.consume(WorthQueryRecoveryResourceTerminal::Expired)
}
