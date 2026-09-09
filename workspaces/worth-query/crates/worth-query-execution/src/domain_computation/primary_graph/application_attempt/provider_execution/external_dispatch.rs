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

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::application_aftermath::{
    dispatch_external_effect, require_fresh_effect_authority, WorthQueryExternalEffectDispatch,
    WorthQueryExternalEffectTransport, WorthQueryPerformedExternalRedispatch,
    WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind,
};
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Why a host could not install an external-effect transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalTransportInstallationDenial {
    /// A transport is already installed; it is never replaced in flight.
    AlreadyInstalled,
}

/// Exact failure before an initial post-commit transport call could be made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchPreparationDenial {
    OwnerReadDenied(
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    AttemptAdmissionDenied,
    CanonicalDerivationDenied,
    TimeObservationDenied,
}

/// Why an admitted re-dispatch could not run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalRedispatchDenial {
    /// Fresh effect authority or current admission failed before transport.
    AdmissionDenied,
    /// The live handle binding carries no co-committed outbox record.
    BindingOutboxMissing,
    /// No host transport is installed on this runtime.
    TransportNotInstalled,
    /// Relational could not establish the exact committed owner row.
    OwnerReadDenied(
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    /// This runtime could not mint a runtime-affine physical attempt.
    AttemptAdmissionDenied,
    /// Canonical derivation for the dispatch event identity failed.
    CanonicalDerivationDenied,
    /// The installed runtime clock could not classify this physical attempt.
    TimeObservationDenied,
}

/// Owner-sealed material for one re-dispatch that actually crossed the
/// runtime's committed-observation dispatch operation.
///
/// The fields and constructor stay private to this module. Other crate
/// siblings may pass the seal to the recovery evidence owner, but cannot mint
/// one from a free handle identity and dispatch.
pub(crate) struct WorthQueryPerformedExternalRedispatchSeal {
    handle: crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
    dispatch: WorthQueryExternalEffectDispatch,
}

struct WorthQueryExternalRedispatchMint;

impl WorthQueryExternalRedispatchMint {
    const fn witness() -> Self {
        Self
    }
}

impl WorthQueryPerformedExternalRedispatchSeal {
    fn new(
        _mint: WorthQueryExternalRedispatchMint,
        handle: crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
        dispatch: WorthQueryExternalEffectDispatch,
    ) -> Self {
        Self { handle, dispatch }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
        WorthQueryExternalEffectDispatch,
    ){
        (self.handle, self.dispatch)
    }
}

impl From<WorthQueryExternalRedispatchDenial> for WorthQueryRecoveryHandleDenial {
    fn from(denial: WorthQueryExternalRedispatchDenial) -> Self {
        match denial {
            WorthQueryExternalRedispatchDenial::AdmissionDenied => {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied,
                )
            }
            WorthQueryExternalRedispatchDenial::BindingOutboxMissing => {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::CorrelationMismatch,
                )
            }
            WorthQueryExternalRedispatchDenial::TransportNotInstalled
            | WorthQueryExternalRedispatchDenial::OwnerReadDenied(_)
            | WorthQueryExternalRedispatchDenial::AttemptAdmissionDenied
            | WorthQueryExternalRedispatchDenial::CanonicalDerivationDenied
            | WorthQueryExternalRedispatchDenial::TimeObservationDenied => {
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::TransitionNotAdmitted,
                )
            }
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Installs the host's external-effect transport, once, for this runtime.
    pub fn install_external_effect_transport(
        &self,
        transport: Arc<dyn WorthQueryExternalEffectTransport>,
    ) -> Result<(), WorthQueryExternalTransportInstallationDenial> {
        self.external_effect_transport
            .set(transport)
            .map_err(|_| WorthQueryExternalTransportInstallationDenial::AlreadyInstalled)
    }

    /// True when this runtime can carry a declared effect out to its rail.
    pub fn has_external_effect_transport(&self) -> bool {
        self.external_effect_transport.get().is_some()
    }

    /// Re-dispatch the handle binding's co-committed outbox through the transport.
    ///
    /// Fresh effect authority is required before any transport call (R8.69). The
    /// outbox is read from the live handle binding — never from a caller-held
    /// receipt copy. Classification stays inside [`dispatch_external_effect`]
    /// (R8.67). The returned proof is privately minted.
    pub fn redispatch_admitted_external_effect<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        authority: &WorthQueryRecoveryEffectAuthority,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryPerformedExternalRedispatch, WorthQueryExternalRedispatchDenial> {
        require_fresh_effect_authority(handle, authority)
            .map_err(|_| WorthQueryExternalRedispatchDenial::AdmissionDenied)?;
        admission
            .validate_current_authority()
            .map_err(|_| WorthQueryExternalRedispatchDenial::AdmissionDenied)?;
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryExternalRedispatchDenial::AdmissionDenied);
        }
        if handle.binding().dispatch_outbox().is_none() {
            return Err(WorthQueryExternalRedispatchDenial::BindingOutboxMissing);
        }
        let committed = self
            .primary_provider
            .committed_dispatch_outbox_for_binding(handle.binding())
            .map_err(WorthQueryExternalRedispatchDenial::OwnerReadDenied)?;
        let Some(transport) = self.external_effect_transport.get() else {
            return Err(WorthQueryExternalRedispatchDenial::TransportNotInstalled);
        };
        let dispatch = self
            .perform_committed_external_dispatch(transport.as_ref(), committed)
            .map_err(|denial| match denial {
                WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied => {
                    WorthQueryExternalRedispatchDenial::AttemptAdmissionDenied
                }
                WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied => {
                    WorthQueryExternalRedispatchDenial::CanonicalDerivationDenied
                }
                WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied => {
                    WorthQueryExternalRedispatchDenial::TimeObservationDenied
                }
                WorthQueryExternalDispatchPreparationDenial::OwnerReadDenied(_) => {
                    unreachable!("owner read occurs before the common dispatch operation")
                }
            })?;
        Ok(WorthQueryPerformedExternalRedispatch::record(
            WorthQueryPerformedExternalRedispatchSeal::new(
                WorthQueryExternalRedispatchMint::witness(),
                handle.authority_identity(),
                dispatch,
            ),
        ))
    }

    pub(super) fn dispatch_committed_external_effect(
        &self,
        outcome: WorthQueryApplicationCommitOutcome,
    ) -> WorthQueryApplicationCommitOutcome {
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            return outcome;
        };
        if receipt.dispatch_outbox().is_none() {
            return WorthQueryApplicationCommitOutcome::Committed(receipt);
        }
        let Some(transport) = self.external_effect_transport.get() else {
            return WorthQueryApplicationCommitOutcome::Committed(receipt);
        };
        let committed = match self.observe_committed_dispatch_outbox(&receipt) {
            Ok(Some(committed)) => committed,
            Ok(None) => return WorthQueryApplicationCommitOutcome::Committed(receipt),
            Err(denial) => {
                return WorthQueryApplicationCommitOutcome::Committed(
                    receipt.with_external_dispatch_preparation_denial(
                        WorthQueryExternalDispatchPreparationDenial::OwnerReadDenied(denial),
                    ),
                );
            }
        };
        match self.perform_committed_external_dispatch(transport.as_ref(), committed) {
            Ok(dispatch) => WorthQueryApplicationCommitOutcome::Committed(
                receipt.with_external_dispatch(dispatch),
            ),
            Err(denial) => WorthQueryApplicationCommitOutcome::Committed(
                receipt.with_external_dispatch_preparation_denial(denial),
            ),
        }
    }

    fn perform_committed_external_dispatch(
        &self,
        transport: &dyn WorthQueryExternalEffectTransport,
        committed: crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<WorthQueryExternalEffectDispatch, WorthQueryExternalDispatchPreparationDenial> {
        let admitted = self
            .admit_external_dispatch_attempt(committed)
            .map_err(|_| WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied)?;
        dispatch_external_effect(transport, admitted).map_err(|denial| match denial {
            crate::domain_computation::application_aftermath::WorthQueryAftermathDerivationFailure::RuntimeTimeUnavailable => {
                WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied
            }
            _ => WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied,
        })
    }
}

#[cfg(test)]
mod safe_retry_affinity_tests;

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::domain_computation::application_aftermath::{
        WorthQueryExternalDispatchRequest, WorthQueryExternalTransportOutcome,
    };
    use crate::domain_computation::primary_graph::recoverable_application_world;

    struct RetryTransport(AtomicUsize);

    impl WorthQueryExternalEffectTransport for RetryTransport {
        fn dispatch(
            &self,
            _request: WorthQueryExternalDispatchRequest<'_>,
        ) -> WorthQueryExternalTransportOutcome {
            match self.0.fetch_add(1, Ordering::AcqRel) {
                0 => WorthQueryExternalTransportOutcome::LostResponse,
                _ => WorthQueryExternalTransportOutcome::Completed,
            }
        }
    }

    #[test]
    fn production_fresh_attempt_operation_distinguishes_safe_redispatch() {
        let (world, receipt) = recoverable_application_world(181, "dispatch-retry");
        let transport = RetryTransport(AtomicUsize::new(0));
        let original_observation = world
            .application
            .observe_committed_dispatch_outbox(&receipt)
            .unwrap()
            .unwrap();
        let retry_observation = original_observation.clone();
        assert_eq!(
            original_observation.record().correlation(),
            retry_observation.record().correlation()
        );
        let original = world
            .application
            .perform_committed_external_dispatch(&transport, original_observation)
            .expect("original dispatch");
        let retry = world
            .application
            .perform_committed_external_dispatch(&transport, retry_observation)
            .expect("safe redispatch");
        assert_eq!(
            original.causal_ladder().emission().identity(),
            retry.causal_ladder().emission().identity()
        );
        assert_ne!(
            original.causal_ladder().attempt().identity(),
            retry.causal_ladder().attempt().identity()
        );
        assert!(retry.is_external_completion());
    }

    #[test]
    fn foreign_owner_observation_denies_before_transport_and_preserves_cause() {
        let (world, local_receipt) = recoverable_application_world(184, "local-dispatch");
        let (foreign_world, foreign_receipt) =
            recoverable_application_world(182, "foreign-dispatch");
        let transport = RetryTransport(AtomicUsize::new(0));
        let local = world
            .application
            .observe_committed_dispatch_outbox(&local_receipt)
            .unwrap()
            .unwrap();
        let foreign = foreign_world
            .application
            .observe_committed_dispatch_outbox(&foreign_receipt)
            .unwrap()
            .unwrap()
            .with_relational_runtime_instance_for_test(local.relational_runtime_instance_id());
        assert_ne!(
            foreign
                .committed_product_publication()
                .product_branch()
                .owner_identity(),
            local
                .committed_product_publication()
                .product_branch()
                .owner_identity()
        );

        assert_eq!(
            world
                .application
                .perform_committed_external_dispatch(&transport, foreign),
            Err(WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied)
        );
        assert_eq!(transport.0.load(Ordering::Acquire), 0);
    }

    #[test]
    fn unavailable_runtime_time_is_a_typed_dispatch_preparation_denial() {
        let (world, receipt) = recoverable_application_world(183, "unavailable-time");
        world.authorization_time.script([]);
        let transport = RetryTransport(AtomicUsize::new(0));
        let observation = world
            .application
            .observe_committed_dispatch_outbox(&receipt)
            .unwrap()
            .unwrap();

        assert_eq!(
            world
                .application
                .perform_committed_external_dispatch(&transport, observation),
            Err(WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied)
        );
        assert_eq!(transport.0.load(Ordering::Acquire), 1);
    }
}
