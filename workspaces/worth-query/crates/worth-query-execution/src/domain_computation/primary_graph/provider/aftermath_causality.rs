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
mod canonical_record;
mod correction_history;
mod lineage_fields;
mod source_fields;
use crate::domain_computation::application_aftermath::{
    WorthQueryCommittedAftermathCausality, WorthQueryPendingAftermathCausality,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryAftermathCausalityLayout;
pub(in crate::domain_computation::primary_graph) use canonical_record::WorthQueryCanonicalCorrectionFact;
pub(in crate::domain_computation::primary_graph) use correction_history::{
    WorthQueryActiveReversalProgress, WorthQueryCorrectionHistoryPageAllocation,
    WorthQueryCorrectionHistoryProgress,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Why an exact canonical aftermath read could not return a complete answer.
///
/// Missing native index generations require explicit reconstruction. A refused
/// page cannot establish active-reversal applicability, and an inconsistent
/// relationship is never resolved by choosing a row from posting order.
pub enum WorthQueryAftermathCausalityReadDenial {
    /// The native snapshot owner could not admit another simultaneous observation.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// Retaining the exact read basis exceeded the native retention capacity.
    RetentionCapacityExhausted,
    /// The native owner could not issue another retention identity.
    RetentionIdentityExhausted,
    /// The native owner could not issue another snapshot identity.
    SnapshotIdentityExhausted,
    /// Preserves the request owner's resource refusal or cancellation/deadline stop.
    ExecutionDenied(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    /// Required canonical records or exact native history could not be resolved.
    Unavailable,
    /// Canonical relationships disagree, including multiple unconsumed reversals.
    InconsistentCanonicalRelationships,
    /// A continuation belongs to a different owner, basis, generation or selector.
    ForeignContinuation,
    /// The selected native index has no installed or exact published generation.
    IndexUnavailable {
        index_id: worth_relational::facade::indexes::DerivedIndexId,
    },
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
    mutation_partition: worth_relational::facade::identity::PartitionId,
) -> MutationIntent {
    let key = pending.key();
    let mut fields = BTreeMap::from([
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
    source_fields::append(layout, pending, &mut fields);
    lineage_fields::append(layout, pending, &mut fields);
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: mutation_partition,
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        pending: &WorthQueryPendingAftermathCausality,
        outcome_identity: Option<WorthQueryApplicationCommitOutcomeIdentity>,
    ) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryAftermathCausalityReadDenial>
    {
        let layout = self.graph.layout.provider_aftermath_causality().clone();
        self.graph.with_runtime_mut(execution, |runtime| {
            self.graph
                .ensure_primary_indexes_for_basis(execution, runtime, basis)
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
            .resolve(execution);
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            resolution
        })
    }

    /// Reads the co-committed relation through an already open snapshot of
    /// `basis`, as publication does once, before the commit's history may
    /// retire.
    pub(in crate::domain_computation::primary_graph) fn resolve_aftermath_causality_in_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        pending: &WorthQueryPendingAftermathCausality,
        outcome_identity: Option<WorthQueryApplicationCommitOutcomeIdentity>,
    ) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryAftermathCausalityReadDenial>
    {
        let layout = self.graph.layout.provider_aftermath_causality().clone();
        self.graph
            .ensure_primary_indexes_for_basis(execution, runtime, basis)
            .map_err(aftermath_index_currency_denial)?;
        WorthQueryAftermathCausalityRead {
            runtime,
            snapshot,
            layout: &layout,
            pending,
            outcome_identity,
        }
        .resolve(execution)
    }
}

fn aftermath_index_currency_denial(
    denial: crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial,
) -> WorthQueryAftermathCausalityReadDenial {
    match denial {
        crate::domain_computation::primary_graph::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(stop) => WorthQueryAftermathCausalityReadDenial::ExecutionDenied(crate::facade::primary_graph::relational_execution_kind(stop, None)),

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
    fn resolve(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<WorthQueryCommittedAftermathCausality>, WorthQueryAftermathCausalityReadDenial>
    {
        let Some(entity_id) = self.lookup_entity(execution)? else {
            return Ok(None);
        };
        let (created_at, values) = self.read_record(execution, entity_id)?;
        if values
            .iter()
            .zip(self.expected_values(&values).iter())
            .any(|(observed, expected)| observed.as_ref() != expected.as_ref())
        {
            return Err("aftermath causality record drifted from admitted meaning".into());
        }
        let committed = self
            .runtime
            .history()
            .immutable_commit_receipt_for_version(created_at)
            .ok_or("aftermath causality creation commit is unavailable")?;
        WorthQueryCommittedAftermathCausality::seal(
            self.pending.clone(),
            committed,
            RecordRef::Entity(entity_id),
        )
        .map(Some)
        .ok_or(WorthQueryAftermathCausalityReadDenial::Unavailable)
    }

    fn lookup_entity(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<worth_relational::facade::identity::EntityId>,
        WorthQueryAftermathCausalityReadDenial,
    > {
        let request = BoundedEntityFieldLookupRequest::new(
            self.snapshot.clone(),
            self.layout.key_index_id,
            self.layout.entity_kind,
            self.layout.key_locator.clone(),
            self.key_value(),
            2,
        )
        .map_err(|_| WorthQueryAftermathCausalityReadDenial::Unavailable)?;
        let lookup = self
            .runtime
            .index_access()
            .execute_bounded_entity_field_lookup(
                request,
                BoundedIndexParityMode::Production,
                execution,
            )
            .map_err(|denied|match denied.kind(){
            worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(cause)=>aftermath_read_stop(cause),
            _=>WorthQueryAftermathCausalityReadDenial::Unavailable,
        })?;
        if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
            return Err(WorthQueryAftermathCausalityReadDenial::Unavailable);
        }
        Ok(lookup.candidate_entity_ids().first().copied())
    }

    fn read_record(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<AftermathCausalityRecord, WorthQueryAftermathCausalityReadDenial> {
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
            .map(|view| {
                view.entity_record_with_projection_scope(execution, entity_id, scope, |record| {
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
                .map_err(aftermath_read_stop)
            })
            .transpose()?
            .flatten()
            .ok_or(WorthQueryAftermathCausalityReadDenial::Unavailable)
    }

    fn expected_values(&self, observed: &AftermathCausalityValues) -> Vec<Option<AspectValue>> {
        let mut values = vec![
            Some(self.key_value()),
            Some(AspectValue::UInt64(self.pending.role().code())),
            Some(AspectValue::String(InternedString::from(
                self.pending.parent().branch_id.0.clone(),
            ))),
            Some(AspectValue::UInt64(self.pending.parent().commit_id.0)),
            Some(AspectValue::UInt64(self.outcome_identity.map_or_else(
                || observed[4].as_ref().and_then(as_u64).unwrap_or(0),
                WorthQueryApplicationCommitOutcomeIdentity::get,
            ))),
        ];
        values.extend(source_fields::values(self.pending));
        values.extend(lineage_fields::values(self.pending));
        values
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
    .chain(source_fields::locators(layout))
    .chain(lineage_fields::locators(layout))
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

fn aftermath_read_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> WorthQueryAftermathCausalityReadDenial {
    WorthQueryAftermathCausalityReadDenial::ExecutionDenied(
        super::relational_execution_denial::relational_execution_kind(cause, None),
    )
}

#[cfg(test)]
mod counted_read_stops;
