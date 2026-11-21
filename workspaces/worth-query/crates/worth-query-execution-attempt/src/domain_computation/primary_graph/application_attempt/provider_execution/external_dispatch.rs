//! Dispatching a co-committed external effect after the commit is durable.
//!
//! Dispatch runs strictly after the mutation transaction has committed, so the
//! outbox record it reads is already recoverable truth. A host that installed
//! no transport, or an operation that declared no external effect, pays nothing
//! and observes nothing here.
//!
//! Safe-retry re-dispatch (Gate 8.7) shares this module so
//! [`dispatch_external_effect`] remains the single classification site (R8.67).

#![deny(private_interfaces)]



use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
pub use worth_query_execution_lineage::facade::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::denial;
pub enum OriginalDispatchObservation<'a> {
    Starting,
    Observed(&'a WorthQueryExternalEffectDispatch),
}



pub use denial::{
    WorthQueryExternalDispatchAttemptDenial, WorthQueryExternalDispatchPreparationDenial,
    WorthQueryExternalRedispatchDenial, WorthQueryExternalTransportInstallationDenial,
};

/// Owner-sealed material for one re-dispatch that actually crossed the
/// runtime's committed-observation dispatch operation.
///
/// The fields and constructor stay private to this module. Other crate
/// siblings may pass the seal to the recovery evidence owner, but cannot mint
/// one from a free handle identity and dispatch.
pub struct WorthQueryPerformedExternalRedispatchSeal {
    handle: crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
    dispatch: WorthQueryExternalEffectDispatch,
}

pub struct WorthQueryExternalRedispatchMint;

impl WorthQueryExternalRedispatchMint {
    pub const fn witness() -> Self {
        Self
    }
}

impl WorthQueryPerformedExternalRedispatchSeal {
    pub fn new(
        _mint: WorthQueryExternalRedispatchMint,
        handle: crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
        dispatch: WorthQueryExternalEffectDispatch,
    ) -> Self {
        Self { handle, dispatch }
    }

    pub fn into_parts(
        self,
    ) -> (
        crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
        WorthQueryExternalEffectDispatch,
    ){
        (self.handle, self.dispatch)
    }
}

pub mod durable_attempt;
