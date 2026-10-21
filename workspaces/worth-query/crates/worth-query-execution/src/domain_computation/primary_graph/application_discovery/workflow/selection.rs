//! One selected lineage exact lookup shared by metadata and instance enumeration.
use super::admission::WorthQueryWorkflowReadDenial as Denial;
use super::text;
use crate::domain_computation::primary_graph::index_currency::ensure_selected_field_indexes_admitted;
use crate::domain_computation::primary_graph::index_currency::SelectedFieldIndexAdmissionStop;
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::BoundedEntityFieldSelectedLookup;
use worth_relational::facade::indexes::BoundedIndexParityMode;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
pub(super) fn lineage<Spec: ApplicationWorkflowSpec>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    basis: &AdmittedRelationalBranchBasis,
    layout: &WorthQueryWorkflowLayout,
    identity: &str,
) -> Result<Option<EntityId>, Denial> {
    ensure_selected_field_indexes_admitted(
        execution,
        runtime,
        basis,
        [Some(layout.lineage.identity_index_id), None],
    )
    .map_err(|stop| match stop {
        SelectedFieldIndexAdmissionStop::Admission(cause) => Denial::Preparation(cause),
        SelectedFieldIndexAdmissionStop::Native => Denial::SelectedIndexUnavailable,
    })?;

    let value = AspectValue::String(InternedString::Raw(identity.to_owned()));
    let view = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or(Denial::BasisUnavailable)?;
    let hit = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            BoundedEntityFieldSelectedLookup::new(
                &view,
                layout.lineage.identity_index_id,
                layout.lineage.entity_kind,
                &layout.lineage.identity,
                &value,
                2,
            ),
            BoundedIndexParityMode::Production,
            execution,
        )
        .map_err(|cause| Denial::Index(cause.kind()))?;
    if hit.overflowed() || hit.candidate_entity_ids().len() > 1 {
        return Err(Denial::Corrupt);
    }
    let Some(lineage) = hit.candidate_entity_ids().first().copied() else {
        return Ok(None);
    };
    let protocol=crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed(runtime,execution,snapshot,lineage,layout.lineage.entity_kind,&layout.lineage.protocol_version,|value|value==Some(&AspectValue::UInt64(crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION))).map_err(Denial::Relational)?;
    if protocol != Some(true) {
        return Err(Denial::Corrupt);
    }
    let pin = runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .map_err(Denial::PinnedRead)?;
    match text::inspect(
        runtime,
        execution,
        &pin,
        lineage,
        layout.lineage.entity_kind,
        &layout.lineage.spec,
        |value| {
            value
                .filter(|value| !value.is_empty())
                .map(|value| value == Spec::IDENTITY.as_str())
        },
    )? {
        Some(true) => Ok(Some(lineage)),
        Some(false) => Ok(None),
        None => Err(Denial::Corrupt),
    }
}
