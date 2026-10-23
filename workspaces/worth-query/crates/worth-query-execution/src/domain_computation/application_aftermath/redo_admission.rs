//! Retirement boundary for provisional redo admission.
//!
//! Proved undo remains descriptive. Retained reapplication requires declared
//! correction selection and fresh ordinary admission; this provisional door
//! preserves typed refusal and recovery custody until its facade is removed.

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryCanonicalWorkEvidence, WorthQueryCanonicalWorkPhases,
};

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryRetainedGovernedInput,
    WorthQuerySelectedProductOperation,
};

use super::governed_input::bound_original_governed_input;
use super::recovery_handle::{RelinquishOnDenial, WorthQueryRecoveryHandleBinding};
use super::recovery_progression::WorthQueryRecoveryEffectAuthority;
use super::redo_denial::{WorthQueryRedoDenial, WorthQueryRedoDenialKind};
use super::redo_intent::{WorthQueryProvedUndo, WorthQueryRedoIntent};
use super::redo_recovery::WorthQueryRedoRecovery;
use super::{WorthQueryAftermathDerivationFailure, WorthQueryPendingAftermathCausality};

impl<Schema> WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    /// Derive a redo intent from this exact retained product occurrence.
    pub fn derive_redo_intent(
        &self,
        proved: &WorthQueryProvedUndo,
    ) -> Result<WorthQueryRedoIntent, WorthQueryAftermathDerivationFailure> {
        let product_commit = self.product().selected_commit().clone();
        let head = self
            .product()
            .relational_basis()
            .observation()
            .commit_receipt()
            .cloned()
            .ok_or(WorthQueryAftermathDerivationFailure::BasisRejected)?;
        WorthQueryRedoIntent::derive(proved, product_commit, head)
    }

    /// Refuse provisional redo after validating its retained custody.
    pub fn admit_redo(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: WorthQueryRedoRecovery,
        authority: &WorthQueryRecoveryEffectAuthority,
        intent: &WorthQueryRedoIntent,
    ) -> Result<WorthQueryRedoAdmission, WorthQueryRedoDenial> {
        // Reading the selected product can itself deny. That is a non-event for
        // the recovery, so it relinquishes rather than dropping the handle it
        // is holding (Q8.21-L11).
        let (recovery, prior_redo) = recovery.admit_deriving(|_| {
            let execution = phase.execution_request_for(&self.application().product_runtime)
                .map_err(|_| WorthQueryRedoDenial::new(super::WorthQueryRedoDenialKind::ForeignPhase))?;
            let pending =
                WorthQueryPendingAftermathCausality::redo_of(intent.undo_commit().clone());
            let prior_redo = self
                .application()
                .primary_provider
                .resolve_aftermath_causality_at_basis(&execution,
                    self.product().relational_basis(),
                    &pending,
                    None,
                )
                .map_err(|denial| match denial {
                    crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::ExecutionDenied(kind) => redo_read_stop(kind),
                    crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    } => WorthQueryRedoDenial::active_snapshot_capacity_exhausted(
                        maximum_active_snapshots,
                    ),
                    crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::Unavailable
                    | crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::ForeignContinuation
                    | crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::IndexUnavailable { .. }
                    | crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::InconsistentCanonicalRelationships => {
                        WorthQueryRedoDenial::stale()
                    }
                    crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionCapacityExhausted => {
                        WorthQueryRedoDenial::retention_capacity_exhausted()
                    }
                    crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::RetentionIdentityExhausted => {
                        WorthQueryRedoDenial::retention_identity_exhausted()
                    }
                    crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial::SnapshotIdentityExhausted => {
                        WorthQueryRedoDenial::snapshot_identity_exhausted()
                    }
                })?
                .map_or(WorthQueryPriorRedoObservation::Absent, |_| {
                    WorthQueryPriorRedoObservation::Committed
                });
            Ok(prior_redo)
        })?;
        refuse_provisional_redo(recovery, authority, intent, prior_redo)
    }
}

/// Admitted redo request ready to re-enter ordinary mutation progression.
#[derive(Debug)]
pub struct WorthQueryRedoAdmission {
    /// Held, not bare: an admitted redo that never reaches
    /// `progress_admitted_redo` — because the host denied on its way there —
    /// consumed nothing (Q8.22-C5).
    recovery: WorthQueryRedoRecovery,
    intent: WorthQueryRedoIntent,
    retained_governed_input: WorthQueryRetainedGovernedInput,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    redo_admission_work: WorthQueryCanonicalWorkEvidence,
    _private: (),
}

impl WorthQueryRedoAdmission {
    pub const fn intent(&self) -> &WorthQueryRedoIntent {
        &self.intent
    }

    pub const fn redo_admission_work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.redo_admission_work
    }

    /// Exact original operation input retained through the redo continuation.
    pub fn original_input<Input: 'static>(&self) -> Option<&Input> {
        self.retained_governed_input.downcast_ref()
    }

    /// Deterministic binding derived from redo intent and original input.
    pub const fn idempotency_binding(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency
    }

    /// Phase bag with the redo_admission slot populated exactly (R8.13 / §8).
    pub fn canonical_work_phases(&self) -> WorthQueryCanonicalWorkPhases {
        WorthQueryCanonicalWorkPhases::new(
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            WorthQueryCanonicalWorkEvidence::zero(),
            self.redo_admission_work,
        )
    }

    pub(crate) fn into_progression_parts(
        self,
    ) -> (
        WorthQueryRedoRecovery,
        WorthQueryRedoIntent,
        WorthQueryRetainedGovernedInput,
        WorthQueryApplicationIdempotencyBinding,
        WorthQueryCanonicalWorkEvidence,
    ) {
        (
            self.recovery,
            self.intent,
            self.retained_governed_input,
            self.idempotency,
            self.redo_admission_work,
        )
    }
}

/// Validate retained evidence, then require the declared correction entrance.
///
/// Copied-intent is detected by re-deriving from `proved` and comparing
/// digests. Duplicate redo is detected from the co-committed Query causal fact. Neither
/// fact is a caller-supplied boolean (R8.43).
pub(super) fn refuse_provisional_redo(
    recovery: WorthQueryRedoRecovery,
    authority: &WorthQueryRecoveryEffectAuthority,
    intent: &WorthQueryRedoIntent,
    prior_redo: WorthQueryPriorRedoObservation,
) -> Result<WorthQueryRedoAdmission, WorthQueryRedoDenial> {
    // Every check below is preparatory — nothing has been redone yet — so a
    // denial relinquishes the handle and leaves the commit recoverable rather
    // than consuming it by omission (Q8.21-L11).
    recovery
        .admit_deriving(|recovery| {
            let handle = recovery.handle();
            authority
                .ensure_for(handle)
                .map_err(|denial| map_recovery_denial(denial.kind()))?;
            reject_copied_intent(intent, recovery.proved())?;
            if prior_redo == WorthQueryPriorRedoObservation::Committed {
                return Err(WorthQueryRedoDenial::duplicate_redo());
            }
            let binding = handle.binding();
            bound_original_governed_input(binding.retained_governed_input())
                .ok_or_else(WorthQueryRedoDenial::changed_operation_meaning)?;
            if binding.runtime_instance_id() != intent.runtime_instance()
                || principal_scope_digest(binding) != *intent.principal_scope_digest()
            {
                return Err(WorthQueryRedoDenial::foreign_principal());
            }
            if binding.installed_operation() != intent.original_operation()
                || binding.installed_aftermath().compatibility_generation()
                    != intent.compatibility_generation()
            {
                return Err(WorthQueryRedoDenial::changed_operation_meaning());
            }
            Err(WorthQueryRedoDenial::new(
                WorthQueryRedoDenialKind::DeclaredDisclosureRequired,
            ))
        })
        .map(|(_, admission)| admission)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryPriorRedoObservation {
    Absent,
    Committed,
}

fn principal_scope_digest(binding: &WorthQueryRecoveryHandleBinding) -> [u8; 32] {
    let scope = binding.principal_scope();
    let principal = scope.principal();
    let mut bytes = [0u8; 32];
    bytes[0..4].copy_from_slice(&principal.partition_id().to_le_bytes());
    bytes[4..12].copy_from_slice(&principal.local_slot().to_le_bytes());
    bytes[12..16].copy_from_slice(&principal.generation().to_le_bytes());
    bytes[16..24].copy_from_slice(&scope.runtime_authority().to_le_bytes());
    bytes
}

fn reject_copied_intent(
    intent: &WorthQueryRedoIntent,
    proved: &WorthQueryProvedUndo,
) -> Result<(), WorthQueryRedoDenial> {
    let expected = WorthQueryRedoIntent::derive(
        proved,
        intent.bound_product_commit().clone(),
        intent.bound_relational_head().clone(),
    )
    .map_err(|_| WorthQueryRedoDenial::stale())?;
    if expected.identity().digest() != intent.identity().digest() {
        return Err(WorthQueryRedoDenial::copied_intent());
    }
    // Binding fields must match the proved undo exactly — a digest collision
    // with drifted fields is still a copy/forge.
    if intent.original_operation() != proved.original_operation()
        || intent.undo_commit() != proved.undo_commit()
        || intent.undo_product_publication() != proved.undo_product_publication()
        || intent.principal_scope_digest() != proved.principal_scope_digest()
        || intent.compatibility_generation() != proved.compatibility_generation()
        || intent.runtime_instance() != proved.runtime_instance()
    {
        return Err(WorthQueryRedoDenial::copied_intent());
    }
    Ok(())
}

pub(crate) fn map_recovery_denial(
    kind: super::recovery_handle::WorthQueryRecoveryHandleDenialKind,
) -> WorthQueryRedoDenial {
    use super::recovery_handle::WorthQueryRecoveryHandleDenialKind as K;
    WorthQueryRedoDenial::new(match kind {
        K::AlreadyTerminal => WorthQueryRedoDenialKind::DuplicateRedo,
        K::Expired => WorthQueryRedoDenialKind::Stale,
        K::FreshAuthorityDenied => WorthQueryRedoDenialKind::NewlyUnauthorized,
        K::ForeignRuntime | K::ForeignPrincipal => WorthQueryRedoDenialKind::ForeignPrincipal,
        K::CompatibilityGenerationMismatch => WorthQueryRedoDenialKind::ChangedOperationMeaning,
        // Redo admission does not execute external redispatch. Request lapses
        // and completion-publication refusals keep their named recovery causes
        // at that separate door; they do not establish redo authority here.
        K::RecoveryNotAdmitted
        | K::RecoveryAlreadyMinted
        | K::SchemaMismatch
        | K::BranchMismatch
        | K::ApplicationBindingGenerationMismatch
        | K::OperationMismatch
        | K::GovernedInputMismatch
        | K::ForeignIdempotencyRead
        | K::CorrelationMismatch
        | K::ForeignBranchEqualOrdinal
        | K::AlreadyCompleted
        | K::CompletionExecutionRequest(_)
        | K::CompletionPublicationPending
        | K::CompletionExecutionDenied { .. }
        | K::CompletionExecutionControlStopped { .. }
        | K::CompletionAllocationDenied { .. }
        | K::CompletionStagingCardinalityOverflow
        | K::CompletionInputDirectoryAllocationDenied { .. }
        | K::TerminalIndexUnavailable
        | K::DispatchOutboxMissing
        | K::TransportNotInstalled
        | K::DispatchOwnerReadDenied(_)
        | K::AttemptAdmissionDenied(_)
        | K::CanonicalDerivationDenied
        | K::TimeObservationDenied
        | K::CompensationNotAdmitted
        | K::ReconciliationNotAdmitted
        | K::AdmissionCancelled
        | K::AdmissionDeadlineExceeded
        | K::AdmissionAuthenticationExpired
        | K::DisclosureAdmissionRequired
        | K::UnresolvedExternalPosture => WorthQueryRedoDenialKind::NewlyUnauthorized,
    })
}

fn redo_read_stop(
    stop: Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
) -> WorthQueryRedoDenial {
    WorthQueryRedoDenial::new(super::WorthQueryRedoDenialKind::ExecutionRequest(stop))
}
