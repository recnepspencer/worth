use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;

pub mod publication_appendix;
pub(super) use publication_appendix::bind_receipt;
pub use publication_appendix::ReceiptAppendix;
pub mod envelope_resolution;
pub mod lookup_key;
pub mod product_affinity;
pub use lookup_key::WorthQueryIdempotencyLookupKey;
pub use product_affinity::WorthQueryProductIdempotencyAffinity;
pub mod recorded_origin;
pub mod registration;
pub mod released_original;
pub use registration::idempotency_create_intent;
pub use released_original::WorthQueryReleasedApplicationEffect;
pub mod snapshot_resolution;
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

pub enum WorthQueryProviderGuardedWorkflowOperationCustody {
    ReleasedOriginal(WorthQueryReleasedApplicationEffect),
    Unseen,
    Readmitted(WorthQueryRecoveredApplicationCommitReceipt),
    IntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
    Indeterminate(WorthQueryProviderIdempotencyResolutionDenial),
}

pub use worth_query_execution_lineage::facade::primary_graph::provider::idempotency::WorthQueryProviderIdempotencyResolutionDenial;

impl WorthQueryPrimaryGraphProvider {
    pub fn resolve_guarded_workflow_operation_custody(
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

    pub fn resolve_idempotency_binding_at_product(
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

    pub fn resolve_application_idempotency(
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
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(stop) => WorthQueryProviderIdempotencyResolutionDenial::ExecutionDenied(crate::domain_computation::primary_graph::relational_execution_kind(stop, None)),

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
