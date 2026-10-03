//! Test-only exact-basis lookup of a canonical completion row by correlation.

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    indexes::{BoundedEntityFieldLookupRequest, BoundedIndexParityMode},
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope, RelationalRuntime},
    storage::RecordLifecycleState,
};

use super::super::super::WorthQueryPrimaryGraphProvider;
use super::{
    decode_completion_values, field, WorthQueryCanonicalCompletionRow,
    WorthQueryInboundCompletionReadDenial,
};
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryInboundCompletionLayout;

impl WorthQueryPrimaryGraphProvider {
    /// A correlation is selected through the Relational owner's exact index
    /// generation. Missing is authoritative only for this admitted basis.
    pub(in crate::domain_computation::primary_graph) fn lookup_inbound_completion_row(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Result<Option<WorthQueryCanonicalCompletionRow>, WorthQueryInboundCompletionReadDenial>
    {
        let layout = self.graph.layout.provider_inbound_completion().clone();
        self.graph
            .with_runtime_mut(|runtime| read_at_basis(runtime, basis, &layout, correlation))
    }
}

fn read_at_basis(
    runtime: &mut RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    layout: &WorthQueryInboundCompletionLayout,
    correlation: &ExternalEffectCorrelationIdentity,
) -> Result<Option<WorthQueryCanonicalCompletionRow>, WorthQueryInboundCompletionReadDenial> {
    use WorthQueryInboundCompletionReadDenial as Denial;
    let snapshot =
        crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
            runtime, basis,
        )
        .map_err(|_| Denial::SnapshotUnavailable)?;
    let result = read_at_snapshot(runtime, &snapshot, layout, correlation);
    crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
    result
}

fn read_at_snapshot(
    runtime: &RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryInboundCompletionLayout,
    correlation: &ExternalEffectCorrelationIdentity,
) -> Result<Option<WorthQueryCanonicalCompletionRow>, WorthQueryInboundCompletionReadDenial> {
    use WorthQueryInboundCompletionReadDenial as Denial;
    let value = AspectValue::String(InternedString::from(hex(correlation.bytes())));
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        layout.correlation_index_id,
        layout.kind,
        layout.correlation.clone(),
        value,
        2,
    )
    .map_err(|_| Denial::IndexUnavailable)?;
    let lookup = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
        .map_err(|_| Denial::IndexUnavailable)?;
    if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
        return Err(Denial::AmbiguousCorrelation);
    }
    let Some(entity_id) = lookup.candidate_entity_ids().first().copied() else {
        return Ok(None);
    };
    let locators = [
        &layout.correlation,
        &layout.family,
        &layout.operation,
        &layout.audience,
        &layout.source,
        &layout.expires_at,
        &layout.key_epoch,
        &layout.message,
        &layout.protocol,
        &layout.version,
        &layout.meaning_digest,
        &layout.payload,
        &layout.original_commit,
        &layout.original_branch,
        &layout.original_entity,
        &layout.original_incarnation,
        &layout.terminal,
        &layout.provenance_kind,
        &layout.transport_attempt,
        &layout.transport_observation,
    ];
    let fields = locators
        .iter()
        .map(|locator| field(locator))
        .collect::<Result<Vec<_>, _>>()?;
    let aspect = layout.correlation.aspect().aspect_key().clone();
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        aspect.clone(),
        fields.clone(),
    )]);
    let (created_at, values) = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .and_then(|view| {
            view.entity_record_with_projection_scope(entity_id, scope, |record| {
                (record.kind_id() == layout.kind
                    && record.lifecycle() == RecordLifecycleState::Live)
                    .then(|| {
                        (
                            record.created_at_version(),
                            fields
                                .iter()
                                .map(|field| record.aspect_field_value(&aspect, field).cloned())
                                .collect::<Option<Vec<_>>>(),
                        )
                    })
            })
        })
        .ok_or(Denial::RowUnavailable)?;
    let values = values.ok_or(Denial::Malformed)?;
    let completed = runtime
        .history()
        .historical_committed_version(created_at)
        .ok_or(Denial::CommitUnavailable)?
        .commit()
        .clone();
    let row = decode_completion_values(runtime, &values, completed)?;
    if row.correlation != *correlation {
        return Err(Denial::Malformed);
    }
    Ok(Some(row))
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
