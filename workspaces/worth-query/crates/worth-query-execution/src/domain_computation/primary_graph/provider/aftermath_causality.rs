//! Relational persistence and owner reads for Query aftermath causal facts.

use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::identity::VersionId;
use worth_relational::facade::indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode};
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, RecordRef,
};

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::{
    WorthQueryCommittedAftermathCausality, WorthQueryPendingAftermathCausality,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryAftermathCausalityLayout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryAftermathCausalityReadDenial {
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    Unavailable,
}

impl From<&'static str> for WorthQueryAftermathCausalityReadDenial {
    fn from(_: &'static str) -> Self {
        Self::Unavailable
    }
}

pub(super) fn aftermath_causality_create_intent(
    layout: &WorthQueryAftermathCausalityLayout,
    pending: &WorthQueryPendingAftermathCausality,
    outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
) -> MutationIntent {
    let key = pending.key();
    let fields = BTreeMap::from([
        (
            layout.key_locator.clone(),
            AspectValue::String(InternedString::from(key.clone())),
        ),
        (
            layout.role_locator.clone(),
            AspectValue::UInt64(pending.role().code()),
        ),
        (
            layout.parent_branch_locator.clone(),
            AspectValue::String(InternedString::from(pending.parent().branch_id.0.clone())),
        ),
        (
            layout.parent_commit_locator.clone(),
            AspectValue::UInt64(pending.parent().commit_id.0),
        ),
        (
            layout.outcome_identity_locator.clone(),
            AspectValue::UInt64(outcome_identity.get()),
        ),
    ]);
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: worth_relational::facade::identity::PartitionId::main(),
        kind_id: layout.entity_kind,
        client_key: worth_relational::facade::symbols::ClientKey::raw(format!(
            "worth-query-aftermath-causality:{key}"
        )),
        fields: AspectFieldPatch::from(fields),
    }))
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation) fn resolve_aftermath_causality_at_basis(
        &self,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        pending: &WorthQueryPendingAftermathCausality,
        outcome_identity: Option<WorthQueryApplicationCommitOutcomeIdentity>,
    ) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryAftermathCausalityReadDenial>
    {
        let layout = self.graph.layout.provider_aftermath_causality().clone();
        self.graph.with_runtime_mut(|runtime| {
            self.graph
                .ensure_primary_indexes_for_basis(runtime, basis)
                .map_err(aftermath_index_currency_denial)?;
            let snapshot = crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, basis)
                .map_err(|denial| aftermath_snapshot_denial(denial.into()))?;
            let resolution = WorthQueryAftermathCausalityRead {
                runtime,
                snapshot: &snapshot,
                layout: &layout,
                pending,
                outcome_identity,
            }
            .resolve();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            resolution.map_err(Into::into)
        })
    }
}

fn aftermath_index_currency_denial(
    denial: crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial,
) -> WorthQueryAftermathCausalityReadDenial {
    match denial {
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted,
        ) => WorthQueryAftermathCausalityReadDenial::RetentionCapacityExhausted,
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted,
        ) => WorthQueryAftermathCausalityReadDenial::RetentionIdentityExhausted,
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted,
        ) => WorthQueryAftermathCausalityReadDenial::SnapshotIdentityExhausted,
        _ => WorthQueryAftermathCausalityReadDenial::Unavailable,
    }
}

fn aftermath_snapshot_denial(
    denial: crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial,
) -> WorthQueryAftermathCausalityReadDenial {
    match denial {
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
            WorthQueryAftermathCausalityReadDenial::RetentionCapacityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
            WorthQueryAftermathCausalityReadDenial::RetentionIdentityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
            WorthQueryAftermathCausalityReadDenial::SnapshotIdentityExhausted
        }
        _ => WorthQueryAftermathCausalityReadDenial::Unavailable,
    }
}

struct WorthQueryAftermathCausalityRead<'a> {
    runtime: &'a worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'a worth_relational::facade::snapshots::SnapshotHandle,
    layout: &'a WorthQueryAftermathCausalityLayout,
    pending: &'a WorthQueryPendingAftermathCausality,
    outcome_identity: Option<WorthQueryApplicationCommitOutcomeIdentity>,
}

impl WorthQueryAftermathCausalityRead<'_> {
    fn resolve(&self) -> Result<Option<WorthQueryCommittedAftermathCausality>, &'static str> {
        let Some(entity_id) = self.lookup_entity()? else {
            return Ok(None);
        };
        let (created_at, values) = self.read_record(entity_id)?;
        if values
            .iter()
            .zip(self.expected_values(&values).iter())
            .any(|(observed, expected)| observed.as_ref() != Some(expected))
        {
            return Err("aftermath causality record drifted from admitted meaning");
        }
        let committed = self
            .runtime
            .history()
            .historical_committed_version(created_at)
            .ok_or("aftermath causality creation commit is unavailable")?;
        WorthQueryCommittedAftermathCausality::seal(
            self.pending.clone(),
            committed.commit().clone(),
            RecordRef::Entity(entity_id),
        )
        .map(Some)
        .ok_or("aftermath causality is not a linear Relational child")
    }

    fn lookup_entity(
        &self,
    ) -> Result<Option<worth_relational::facade::identity::EntityId>, &'static str> {
        let request = BoundedEntityFieldLookupRequest::new(
            self.snapshot.clone(),
            self.layout.key_index_id,
            self.layout.entity_kind,
            self.layout.key_locator.clone(),
            self.key_value(),
            2,
        )
        .map_err(|_| "aftermath causality lookup request was rejected")?;
        let lookup = self
            .runtime
            .index_access()
            .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
            .map_err(|_| "aftermath causality index lookup failed")?;
        if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
            return Err("aftermath causality key is not unique");
        }
        Ok(lookup.candidate_entity_ids().first().copied())
    }

    fn read_record(
        &self,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<AftermathCausalityRecord, &'static str> {
        let fields = required_fields(self.layout)?;
        let aspect = self.layout.key_locator.aspect().aspect_key().clone();
        let scope =
            ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
                aspect.clone(),
                fields.clone(),
            )]);
        self.runtime
            .read_truth()
            .project_snapshot(self.snapshot)
            .and_then(|view| {
                view.entity_record_with_projection_scope(entity_id, scope, |record| {
                    (record.kind_id() == self.layout.entity_kind
                        && record.lifecycle() == RecordLifecycleState::Live)
                        .then(|| {
                            let values = fields
                                .iter()
                                .map(|field| record.aspect_field_value(&aspect, field).cloned())
                                .collect();
                            (record.created_at_version(), values)
                        })
                })
            })
            .ok_or("aftermath causality record is not authoritative")
    }

    fn expected_values(&self, observed: &AftermathCausalityValues) -> [AspectValue; 5] {
        [
            self.key_value(),
            AspectValue::UInt64(self.pending.role().code()),
            AspectValue::String(InternedString::from(
                self.pending.parent().branch_id.0.clone(),
            )),
            AspectValue::UInt64(self.pending.parent().commit_id.0),
            AspectValue::UInt64(self.outcome_identity.map_or_else(
                || observed[4].as_ref().and_then(as_u64).unwrap_or(0),
                WorthQueryApplicationCommitOutcomeIdentity::get,
            )),
        ]
    }

    fn key_value(&self) -> AspectValue {
        AspectValue::String(InternedString::from(self.pending.key()))
    }
}

type AftermathCausalityValues = Vec<Option<AspectValue>>;
type AftermathCausalityRecord = (VersionId, AftermathCausalityValues);

fn required_fields(
    layout: &WorthQueryAftermathCausalityLayout,
) -> Result<Vec<worth_foundational::facade::FieldKey>, &'static str> {
    [
        &layout.key_locator,
        &layout.role_locator,
        &layout.parent_branch_locator,
        &layout.parent_commit_locator,
        &layout.outcome_identity_locator,
    ]
    .into_iter()
    .map(|locator| {
        locator
            .field_path()
            .fields()
            .first()
            .cloned()
            .ok_or("aftermath causality field locator is empty")
    })
    .collect()
}

fn as_u64(value: &AspectValue) -> Option<u64> {
    match value {
        AspectValue::UInt64(value) => Some(*value),
        _ => None,
    }
}
