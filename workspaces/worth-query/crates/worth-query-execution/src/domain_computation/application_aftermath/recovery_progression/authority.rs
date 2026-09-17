//! Runtime-and-handle-affine recovery authority (R8.31 / Q8.20).
//!
//! A registry slot is only lifecycle-local addressing. Recovery authority is
//! bound to both the concrete runtime that admitted current truth and the exact
//! handle it admitted, so neither cross-runtime nor same-runtime substitution
//! can open a transition.

use worth_proof::{
    Artifact, AssumptionBasis, CurrentValidity, FreshnessScopedBasis, NoProofs, PhaseMarker,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

use super::super::recovery_handle::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleAuthorityIdentity,
    WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
};
use super::super::WorthQueryUndoAdmission;
use super::disclose::WorthQueryRecoveryDisclosureAdmission;
use super::expiry::deny_if_expired;

#[derive(Debug)]
struct WorthQueryCurrentRecoveryAuthorityPhase;
impl PhaseMarker for WorthQueryCurrentRecoveryAuthorityPhase {}

worth_proof::authority_marker!(WorthQueryRecoveryCurrentAuthorityMint);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorthQueryRecoveryAuthorityBasis {
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
    pub(crate) fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
    ) -> Self {
        Self {
            current: Artifact::with_current_basis(
                (),
                WorthQueryRecoveryAuthorityBasis {
                    runtime: runtime_authority,
                    handle: handle_authority,
                },
                WorthQueryRecoveryCurrentAuthorityMint::witness(),
            ),
        }
    }

    pub(crate) fn ensure_for(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        handle.ensure_live()?;
        deny_if_expired(handle, handle.registry_clock())?;
        ensure_authority_owns_handle(*self.current.strong_basis().value(), handle)
    }

    /// Finalize an already performed dispatch without resampling expiry after I/O.
    pub(crate) fn ensure_performed_for(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        handle.ensure_live()?;
        ensure_authority_owns_handle(*self.current.strong_basis().value(), handle)
    }
}

/// Proof that disclosure admission and binding truth matched for inspection.
#[derive(Debug)]
pub struct WorthQueryRecoveryInspectAuthority {
    current: WorthQueryCurrentRecoveryAuthority,
}

impl WorthQueryRecoveryInspectAuthority {
    pub(crate) fn mint(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        handle_authority: WorthQueryRecoveryHandleAuthorityIdentity,
    ) -> Self {
        Self {
            current: Artifact::with_current_basis(
                (),
                WorthQueryRecoveryAuthorityBasis {
                    runtime: runtime_authority,
                    handle: handle_authority,
                },
                WorthQueryRecoveryCurrentAuthorityMint::witness(),
            ),
        }
    }

    pub(crate) fn ensure_for(
        &self,
        handle: &WorthQueryRecoveryHandle,
    ) -> Result<(), WorthQueryRecoveryHandleDenial> {
        handle.ensure_live()?;
        deny_if_expired(handle, handle.registry_clock())?;
        ensure_authority_owns_handle(*self.current.strong_basis().value(), handle)
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

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Re-read current truth and mint effect authority owned by this runtime.
    pub fn admit_recovery_effect_authority<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandleDenial> {
        handle.ensure_live()?;
        let owner = self.runtime.authority_identity();
        ensure_same_runtime(owner, handle.runtime_authority())?;
        ensure_same_runtime(owner, admission.runtime_authority())?;
        deny_if_expired(handle, &self.authorization_clock)?;
        super::binding_axis::check_binding_axes(handle.binding(), admission)?;
        Ok(WorthQueryRecoveryEffectAuthority::mint(
            self.runtime.authority_identity(),
            handle.authority_identity(),
        ))
    }

    /// Re-read current truth for the exact recovery held by an admitted undo.
    pub fn admit_undo_recovery_effect_authority<Operation, Input, Scope>(
        &self,
        undo: &WorthQueryUndoAdmission,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandleDenial> {
        self.admit_recovery_effect_authority(undo.recovery_handle(), admission)
    }

    /// Re-read current truth and mint disclosure-backed inspection authority.
    pub fn admit_recovery_inspect_authority<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        disclosure: &WorthQueryRecoveryDisclosureAdmission,
    ) -> Result<WorthQueryRecoveryInspectAuthority, WorthQueryRecoveryHandleDenial> {
        handle.ensure_live()?;
        let owner = self.runtime.authority_identity();
        disclosure.ensure_for_runtime(owner)?;
        ensure_same_runtime(owner, handle.runtime_authority())?;
        ensure_same_runtime(owner, admission.runtime_authority())?;
        deny_if_expired(handle, &self.authorization_clock)?;
        super::binding_axis::check_binding_axes(handle.binding(), admission)?;
        Ok(WorthQueryRecoveryInspectAuthority::mint(
            self.runtime.authority_identity(),
            handle.authority_identity(),
        ))
    }
}

fn ensure_authority_owns_handle(
    basis: WorthQueryRecoveryAuthorityBasis,
    handle: &WorthQueryRecoveryHandle,
) -> Result<(), WorthQueryRecoveryHandleDenial> {
    if handle.runtime_authority() != basis.runtime || handle.authority_identity() != basis.handle {
        return Err(WorthQueryRecoveryHandleDenial::new(
            WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied,
        ));
    }
    Ok(())
}

/// Admission-time ownership: the material presented to *obtain* authority must
/// come from this runtime.
///
/// Deliberately a different cause from [`ensure_authority_owns_handle`], which
/// runs at proof *use*. `ForeignRuntime` means "this handle or this admission
/// was never ours"; `FreshAuthorityDenied` means "this authority is ours but is
/// not the authority for this handle". Collapsing them would let a caller learn
/// only that something was refused (Q8.20-C4).
///
/// Takes the two identities rather than the runtime so the distinction is
/// testable without standing up a published runtime — the caller still reads
/// the owner identity from the runtime it is a method on, never from an
/// argument.
pub(super) fn ensure_same_runtime(
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
