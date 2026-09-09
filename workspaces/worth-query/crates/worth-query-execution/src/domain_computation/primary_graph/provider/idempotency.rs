use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent,
};

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphCommittedApplication;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout;

mod product_affinity;
pub(in crate::domain_computation::primary_graph) use product_affinity::WorthQueryProductIdempotencyAffinity;
mod snapshot_resolution;
use snapshot_resolution::resolve_at_snapshot;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProviderIdempotencyResolution {
    Absent,
    Equivalent(WorthQueryPrimaryGraphCommittedApplication),
    Drift,
    Unpublished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProviderIdempotencyResolutionDenial
{
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    Unavailable,
}

impl From<&'static str> for WorthQueryProviderIdempotencyResolutionDenial {
    fn from(_: &'static str) -> Self {
        Self::Unavailable
    }
}

pub(super) fn idempotency_create_intent(
    layout: &WorthQueryProviderIdempotencyLayout,
    binding: WorthQueryApplicationIdempotencyBinding,
    outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
    emitted_effect_count: u64,
) -> MutationIntent {
    let key = binding.key_text();
    let fields = BTreeMap::from([
        (
            layout.key_locator.clone(),
            AspectValue::String(InternedString::from(key.clone())),
        ),
        (
            layout.intent_locator.clone(),
            AspectValue::String(InternedString::from(binding.intent_text())),
        ),
        (
            layout.outcome_identity_locator.clone(),
            AspectValue::UInt64(outcome_identity.get()),
        ),
        (
            layout.emitted_effect_count_locator.clone(),
            AspectValue::UInt64(emitted_effect_count),
        ),
    ]);
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: worth_relational::facade::identity::PartitionId::main(),
        kind_id: layout.entity_kind,
        client_key: worth_relational::facade::symbols::ClientKey::raw(format!(
            "worth-query-idempotency:{key}"
        )),
        fields: AspectFieldPatch::from(fields),
    }))
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn resolve_idempotency_binding_at_product(
        &self,
        binding: WorthQueryApplicationIdempotencyBinding,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    ) -> Result<
        WorthQueryProviderIdempotencyResolution,
        WorthQueryProviderIdempotencyResolutionDenial,
    > {
        self.application_attempt_work.observe_retained_resolution();
        let layout = self.graph.layout.provider_idempotency().clone();
        let product_affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        self.graph.with_runtime_mut(|runtime| {
            self.resume_pending_application_publication(runtime)
                .map_err(pending_publication_denial)?;
            let basis = product.observation().basis().relational_basis();
            if let Some(resolution) = self.resolve_unpublished_application_idempotency(
                &product_affinity,
                binding,
            ) {
                return Ok(resolution);
            }
            if let Some(resolution) = self.resolve_completed_application_idempotency(
                &product_affinity,
                binding,
            ) {
                return Ok(resolution);
            }
            self.graph
                .ensure_primary_indexes_for_basis(runtime, basis)
                .map_err(idempotency_index_currency_denial)?;
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                .map_err(|denial| idempotency_snapshot_denial(denial.into()))?;
            let resolution = resolve_at_snapshot(
                self,
                runtime,
                &snapshot,
                &layout,
                &product_affinity,
                binding,
            );
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            let resolution = resolution.map_err(WorthQueryProviderIdempotencyResolutionDenial::from)?;
            if let WorthQueryProviderIdempotencyResolution::Equivalent(committed) = &resolution {
                self.repair_equivalent_publication_settlement(runtime, committed)?;
            }
            Ok(resolution)
        })
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_application_idempotency(
        &self,
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
        basis.resolve(self)
    }
}

fn pending_publication_denial(
    failure: crate::domain_computation::WorthQueryProviderSessionFailure,
) -> WorthQueryProviderIdempotencyResolutionDenial {
    match failure.kind() {
        crate::domain_computation::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryProviderIdempotencyResolutionDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::RetentionCapacityExhausted
        }
        crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted
        }
        crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted => {
            WorthQueryProviderIdempotencyResolutionDenial::SnapshotIdentityExhausted
        }
        _ => WorthQueryProviderIdempotencyResolutionDenial::Unavailable,
    }
}

fn idempotency_index_currency_denial(
    denial: crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial,
) -> WorthQueryProviderIdempotencyResolutionDenial {
    match denial {
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
