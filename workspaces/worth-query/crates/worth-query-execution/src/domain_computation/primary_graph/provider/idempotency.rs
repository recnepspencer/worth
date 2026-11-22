use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;

mod publication_appendix;
pub(super) use publication_appendix::bind_receipt;
pub(in crate::domain_computation::primary_graph) use publication_appendix::ReceiptAppendix;
pub(in crate::domain_computation::primary_graph) mod conditional_key;
pub(in crate::domain_computation::primary_graph) use conditional_key::WorthQueryConditionalKeyFate;
mod envelope_resolution;
mod lookup_key;
pub(crate) mod product_affinity;
pub(in crate::domain_computation::primary_graph) use lookup_key::WorthQueryIdempotencyLookupKey;
pub(in crate::domain_computation::primary_graph) use product_affinity::WorthQueryProductIdempotencyAffinity;
pub(super) mod recorded_origin;
mod registration;
pub(super) mod released_original;
pub(super) use registration::ORIGINAL_CLIENT_KEY_PREFIX;
pub use registration::idempotency_create_intent;
pub use released_original::WorthQueryReleasedApplicationEffect;
pub(crate) mod snapshot_resolution;
use snapshot_resolution::resolve_at_snapshot;
pub use snapshot_resolution::{
    WorthQueryRecoveredAftermathCausality, WorthQueryRecoveredApplicationCommitReceipt,
    WorthQueryRecoveredDispatchOutbox,
};

#[derive(Debug)]
pub enum WorthQueryProviderIdempotencyResolution {
    ReleasedOriginal(WorthQueryReleasedApplicationEffect),
    PreparationBusy,
    RecoveredUnpublished {
        handle: worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
    },
    Absent,
    Readmitted(WorthQueryRecoveredApplicationCommitReceipt),
    Drift,
    Unpublished,
    PreparationPending {
        handle: worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
        stage: crate::domain_computation::WorthQueryProductPreparationStage,
    },
}

pub(in crate::domain_computation::primary_graph) enum WorthQueryProviderGuardedWorkflowOperationCustody
{
    ReleasedOriginal(WorthQueryReleasedApplicationEffect),
    Unseen,
    Readmitted(WorthQueryRecoveredApplicationCommitReceipt),
    IntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
    Indeterminate(WorthQueryProviderIdempotencyResolutionDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderIdempotencyResolutionDenial {
    ForeignAffinity,
    PreparationBusy,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    /// The key's first-encoding record matches every durable part of this
    /// intent but names its operation by a seal no later runtime can confirm.
    RecordedIntentUnverifiable,
    ExecutionDenied(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    NativeKeyLookup {
        index: worth_relational::facade::indexes::DerivedIndexId,
        cause: worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind,
    },
    Unavailable,
}

impl From<&'static str> for WorthQueryProviderIdempotencyResolutionDenial {
    fn from(_: &'static str) -> Self {
        Self::Unavailable
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn resolve_guarded_workflow_operation_custody(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryApplicationIdempotencyBinding,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> WorthQueryProviderGuardedWorkflowOperationCustody {
        use WorthQueryProviderGuardedWorkflowOperationCustody as Custody;

        let affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        match self.resolve_unpublished_application_idempotency(&affinity, binding) {
            Some(WorthQueryProviderIdempotencyResolution::ReleasedOriginal(original)) => {
                return Custody::ReleasedOriginal(original)
            }
            Some(WorthQueryProviderIdempotencyResolution::PreparationBusy) => {
                return Custody::Indeterminate(
                    WorthQueryProviderIdempotencyResolutionDenial::PreparationBusy,
                )
            }
            Some(WorthQueryProviderIdempotencyResolution::Drift) => return Custody::IntentDrift,
            None
            | Some(
                WorthQueryProviderIdempotencyResolution::Absent
                | WorthQueryProviderIdempotencyResolution::Readmitted(_)
                | WorthQueryProviderIdempotencyResolution::RecoveredUnpublished { .. }
                | WorthQueryProviderIdempotencyResolution::Unpublished
                | WorthQueryProviderIdempotencyResolution::PreparationPending { .. },
            ) => {}
        }
        // A retained World partial keeps its publication slot reserved for
        // resumption, and a reserved slot reports its binding, so exact
        // unpublished custody is read before that reservation.
        if let Some((recorded, handle)) =
            self.inspect_unpublished_application_idempotency(&affinity, binding)
        {
            return if recorded == binding {
                Custody::ProductUnpublished(handle)
            } else {
                Custody::IntentDrift
            };
        }
        let pending = self.inspect_pending_application_idempotency(affinity.incarnation());
        if let Some(Some(recorded)) = pending
            && recorded.key_identity() == binding.key_identity()
        {
            return if recorded == binding {
                Custody::PublicationPending
            } else {
                Custody::IntentDrift
            };
        }
        if let Some(None) = pending {
            return Custody::Indeterminate(
                WorthQueryProviderIdempotencyResolutionDenial::Unavailable,
            );
        }
        match self.resolve_idempotency_binding_at_product(execution, binding, product) {
            Ok(WorthQueryProviderIdempotencyResolution::PreparationBusy) => Custody::Indeterminate(
                WorthQueryProviderIdempotencyResolutionDenial::PreparationBusy,
            ),
            Ok(WorthQueryProviderIdempotencyResolution::ReleasedOriginal(original)) => {
                Custody::ReleasedOriginal(original)
            }
            Ok(WorthQueryProviderIdempotencyResolution::Absent) => Custody::Unseen,
            Ok(WorthQueryProviderIdempotencyResolution::Readmitted(receipt)) => {
                Custody::Readmitted(receipt)
            }
            Ok(WorthQueryProviderIdempotencyResolution::Drift) => Custody::IntentDrift,
            Ok(WorthQueryProviderIdempotencyResolution::PreparationPending { .. }) => {
                Custody::PublicationPending
            }
            Ok(WorthQueryProviderIdempotencyResolution::RecoveredUnpublished { handle }) => {
                Custody::ProductUnpublished(handle)
            }
            Ok(WorthQueryProviderIdempotencyResolution::Unpublished) => {
                Custody::Indeterminate(WorthQueryProviderIdempotencyResolutionDenial::Unavailable)
            }
            Err(denial) => Custody::Indeterminate(denial),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_idempotency_binding_at_product(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryApplicationIdempotencyBinding,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Result<
        WorthQueryProviderIdempotencyResolution,
        WorthQueryProviderIdempotencyResolutionDenial,
    > {
        let candidates = crate::domain_computation::primary_graph::application_attempt::idempotency::envelope_candidates::EnvelopeCandidates::new(std::slice::from_ref(&binding))?;
        self.resolve_envelope_idempotency_at_product(
            execution,
            &candidates,
            product,
            crate::domain_computation::primary_graph::application_contribution::authored_edit::AuthoredEnvelopeResolutionPurpose::Execute,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_application_idempotency(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider_session: &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding,
    ) -> Result<
        WorthQueryProviderIdempotencyResolution,
        WorthQueryProviderIdempotencyResolutionDenial,
    > {
        let basis = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .idempotency_basis(provider_session)
            .ok_or(WorthQueryProviderIdempotencyResolutionDenial::Unavailable)?;
        basis.resolve(execution, self)
    }

    /// Observe the selected product occurrence's current Relational basis and
    /// run `read` against its prepared key index and exact-basis snapshot with
    /// guaranteed snapshot release. Shared by ordinary envelope resolution and
    /// conditional key classification.
    pub(super) fn read_current_idempotency_at_basis<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        read: impl FnOnce(
            &worth_relational::facade::snapshots::SnapshotHandle,
        ) -> Result<T, WorthQueryProviderIdempotencyResolutionDenial>,
    ) -> Result<T, WorthQueryProviderIdempotencyResolutionDenial> {
        self.graph
            .carry_primary_index_basis_for_basis(execution, runtime, basis)
            .map_err(idempotency_index_currency_denial)?;
        if !crate::domain_computation::primary_graph::index_currency::selected_index_is_current(
            execution,
            runtime,
            basis,
            layout.key_index_id,
        )
        .map_err(snapshot_resolution::idempotency_read_stop)?
        {
            // A carried known-empty selection is not yet a concrete lookup
            // generation. Admit only this key index through Native's ordinary
            // field preparation; it never reconstructs a missing population.
            runtime
                .index_authority()
                .ensure_field_index_for_basis(
                    layout.key_index_id,
                    basis,
                    crate::domain_computation::primary_graph::index_maintenance_budget::ordinary_index_maintenance_budget(),
                    execution,
                )
                .map_err(|denial| match denial.kind {
                    worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::Execution(
                        cause,
                    ) => snapshot_resolution::idempotency_read_stop(cause),
                    _ => WorthQueryProviderIdempotencyResolutionDenial::Unavailable,
                })?;
        }
        let snapshot =
            crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                runtime, basis,
            )
            .map_err(|denial| idempotency_snapshot_denial(denial.into()))?;
        let result = read(&snapshot);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        result
    }
}

pub(super) fn observe_selected_idempotency_branch(
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
) -> Result<
    worth_runtime_world::facade::ProductBranchObservation,
    WorthQueryProviderIdempotencyResolutionDenial,
> {
    product
        .observation_port()
        .observe_product_branch_occurrence(product.observation().lifecycle_incarnation())
        .map_err(|_| WorthQueryProviderIdempotencyResolutionDenial::ForeignAffinity)
}

fn pending_publication_denial(
    failure: crate::domain_computation::WorthQueryProviderSessionFailure,
) -> WorthQueryProviderIdempotencyResolutionDenial {
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
    match super::relational_execution_denial::native_preparation_kind(&failure) {
        Ok(Some(kind)) => {
            return WorthQueryProviderIdempotencyResolutionDenial::ExecutionDenied(Ok(kind))
        }
        Err(stop) => {
            return WorthQueryProviderIdempotencyResolutionDenial::ExecutionDenied(Err(stop))
        }
        Ok(None) => {}
    }
    match failure.kind() {
        Kind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryProviderIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Kind::RetentionCapacityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted
        }
        Kind::RetentionIdentityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted
        }
        Kind::SnapshotIdentityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted
        }
        kind @ (Kind::StaleIncarnation
        | Kind::ProjectionContract { .. }
        | Kind::HistoricalVisibility { .. }
        | Kind::OutputCorrespondenceMismatch { .. }
        | Kind::ContentInconsistency
        | Kind::Content(_)
        | Kind::StorageUnavailable(_)
        | Kind::ExecutionResource { .. }
        | Kind::ExecutionNestedPatternStopped { .. }
        | Kind::ExecutionWorkerPanicked { .. }
        | Kind::ExecutionIdentitiesNotCanonical { .. }
        | Kind::ExecutionUncheckedCustomKernel { .. }) => {
            WorthQueryProviderIdempotencyResolutionDenial::ExecutionDenied(Ok(kind))
        }
        Kind::AllocationDenied
        | Kind::ForeignOperationAttempt
        | Kind::ForeignExecutionBasis
        | Kind::ForeignGraphAuthority
        | Kind::UndeclaredOperationScope
        | Kind::ResourceEnvelopeMismatch
        | Kind::CandidateIdentityExhausted
        | Kind::IndexMaintenanceBudgetExceeded
        | Kind::IndexGenerationIdentityExhausted
        | Kind::ProviderIdentityMismatch
        | Kind::ProviderGenerationMismatch
        | Kind::SessionProtocolUnsupported
        | Kind::ProviderRejected
        | Kind::ProviderPanicked
        | Kind::TokenNotMintedForPlan
        | Kind::EmptyPhysicalSessionIdentity
        | Kind::SessionIdentityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::Unavailable
        }
    }
}

fn idempotency_index_currency_denial(
    denial: crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial,
) -> WorthQueryProviderIdempotencyResolutionDenial {
    match denial {
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(stop) => WorthQueryProviderIdempotencyResolutionDenial::ExecutionDenied(crate::facade::primary_graph::relational_execution_kind(stop, None)),

        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted,
        ) => WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted,
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted,
        ) => WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted,
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted,
        ) => WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted,
        _ => WorthQueryProviderIdempotencyResolutionDenial::Unavailable,
    }
}

fn idempotency_snapshot_denial(
    denial: crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial,
) -> WorthQueryProviderIdempotencyResolutionDenial {
    match denial {
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryProviderIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted
        }
        _ => WorthQueryProviderIdempotencyResolutionDenial::Unavailable,
    }
}

#[cfg(test)]
#[path = "idempotency/denial_mapping_tests.rs"]
mod denial_mapping_tests;
