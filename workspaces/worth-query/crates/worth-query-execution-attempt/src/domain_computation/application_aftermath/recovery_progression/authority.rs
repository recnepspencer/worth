//! Runtime-and-handle-affine recovery authority (R8.31 / Q8.20).
//!
//! A registry slot is only lifecycle-local addressing. Recovery authority is
//! bound to both the concrete runtime that admitted current truth and the exact
//! handle it admitted, so neither cross-runtime nor same-runtime substitution
//! can open a transition.

use worth_proof::{
    Artifact, AssumptionBasis, CurrentValidity, FreshnessScopedBasis, NoProofs, PhaseMarker,
};
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use super::super::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleAuthorityIdentity,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use super::expiry::deny_if_expired;

#[derive(Debug)]
struct WorthQueryCurrentRecoveryAuthorityPhase;
impl PhaseMarker for WorthQueryCurrentRecoveryAuthorityPhase {}

worth_proof::authority_marker!(WorthQueryRecoveryCurrentAuthorityMint);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorthQueryRecoveryAuthorityBasis {
    home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    runtime: WorthQueryRuntimeAuthorityIdentity,
    handle: WorthQueryRecoveryHandleAuthorityIdentity,
}

type WorthQueryCurrentRecoveryAuthority = Artifact<
    WorthQueryCurrentRecoveryAuthorityPhase,
    (),
    NoProofs,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<WorthQueryRecoveryAuthorityBasis>>,
>;

/// Proof that current capability, operation, and binding truth matched.
///
/// Only the owning runtime can mint this proof. The carried basis identifies
/// that runtime without exposing a caller-constructible identity lane.
#[derive(Debug)]
pub struct WorthQueryRecoveryEffectAuthority {
    current: WorthQueryCurrentRecoveryAuthority,
}

impl WorthQueryRecoveryEffectAuthority {
    pub fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
        home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Self {
        Self {
            current: Artifact::with_current_basis(
                (),
                WorthQueryRecoveryAuthorityBasis {
                    runtime: runtime_authority,
                    home_owner,
                    handle: handle_authority,
                },
                WorthQueryRecoveryCurrentAuthorityMint::witness(),
            ),
        }
    }

    pub fn ensure_for(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        ensure_authority_owns_handle(*self.current.strong_basis().value(), handle)?;
        handle.ensure_live()?;
        deny_if_expired(handle, handle.registry_clock())
    }

    /// Finalize an already performed dispatch without resampling expiry after I/O.
    pub fn ensure_performed_for(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        ensure_authority_owns_handle(*self.current.strong_basis().value(), handle)?;
        handle.ensure_live()
    }
}

/// Proof that disclosure admission and binding truth matched for inspection.
#[derive(Debug)]
pub struct WorthQueryRecoveryInspectAuthority {
    current: WorthQueryCurrentRecoveryAuthority,
}

impl WorthQueryRecoveryInspectAuthority {
    pub fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
        home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Self {
        Self {
            current: Artifact::with_current_basis(
                (),
                WorthQueryRecoveryAuthorityBasis {
                    runtime: runtime_authority,
                    home_owner,
                    handle: handle_authority,
                },
                WorthQueryRecoveryCurrentAuthorityMint::witness(),
            ),
        }
    }

    pub fn ensure_for(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        ensure_authority_owns_handle(*self.current.strong_basis().value(), handle)?;
        handle.ensure_live()?;
        deny_if_expired(handle, handle.registry_clock())
    }
}

/// Re-establish that effect authority belongs to this exact live handle.
pub fn require_fresh_effect_authority(
    handle: &WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryEffectAuthority,
) -> Result<(), WorthQueryRecoveryHandleDenial> {
    authority.ensure_for(handle)
}

/// Inspection requires disclosure-backed authority for this exact live handle.
pub fn require_inspect_disclosure(
    handle: &WorthQueryRecoveryHandle,
    authority: &WorthQueryRecoveryInspectAuthority,
) -> Result<(), WorthQueryRecoveryHandleDenial> {
    authority.ensure_for(handle)
}


fn ensure_authority_owns_handle(
    basis: WorthQueryRecoveryAuthorityBasis,
    handle: &WorthQueryRecoveryHandle,
) -> Result<(), WorthQueryRecoveryHandleDenial> {
    handle
        .home_owner()
        .admit_incarnation(basis.home_owner)
        .map_err(|cause| {
            WorthQueryRecoveryHandleDenial::new(match cause {
                worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                    WorthQueryRecoveryHandleDenialKind::StaleIncarnation
                }
                worth_proof::HomeIncarnationDenial::ForeignHome => {
                    WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
                }
            })
        })?;
    if handle.runtime_authority() != basis.runtime || handle.authority_identity() != basis.handle {
        return Err(WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied,
        ));
    }
    Ok(())
}

pub fn ensure_same_runtime(
    owner: WorthQueryRuntimeAuthorityIdentity,
    presented: WorthQueryRuntimeAuthorityIdentity,
) -> Result<(), WorthQueryRecoveryHandleDenial> {
    if presented != owner {
        return Err(WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::ForeignRuntime,
        ));
    }
    Ok(())
}

mod court_doors_for_test;
