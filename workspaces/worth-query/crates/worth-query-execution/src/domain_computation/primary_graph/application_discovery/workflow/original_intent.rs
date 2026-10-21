//! Original native start-intent selection at an explicitly retained data occurrence.
//! Consumes the original opaque native start-intent header, never reconstructed IDs.
use super::admission::admit;
use super::admission::WorthQueryWorkflowReadDenial as Denial;
use crate::domain_computation::primary_graph::application_attempt::observe_adjacency;
use crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowInstanceDiscoveryIntent;
use crate::domain_computation::primary_graph::index_currency::ensure_selected_field_indexes_admitted;
use crate::domain_computation::primary_graph::index_currency::SelectedFieldIndexAdmissionStop;

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadBinding;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadRoute;
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledWorkflowReadBinding;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::BoundedEntityFieldSelectedLookup;
use worth_relational::facade::indexes::BoundedIndexParityMode;

/// Private data locator borrows the exact selected occurrence, never a grant.
pub(super) struct OriginalWorkflowCandidate<'data, 'runtime, Schema> {
    pub(super) data: &'data WorthQuerySelectedProductOperation<'runtime, Schema>,
    pub(super) instance: EntityId,
    pub(super) lineage: EntityId,
    pub(super) subject: EntityId,
}

type AdmittedOriginalWorkflowCandidate<'data, 'runtime, Schema, B> = (
    OriginalWorkflowCandidate<'data, 'runtime, Schema>,
    crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
        Schema,
        <B as ApplicationWorkflowReadBinding<Schema>>::Capability,
        <B as ApplicationWorkflowReadBinding<Schema>>::Operation,
        <B as ApplicationWorkflowReadBinding<Schema>>::Input,
    >,
);

pub(super) fn lookup<'data, 'runtime, Schema, Spec, B>(
    current: &WorthQuerySelectedProductOperation<'_, Schema>,
    data: &'data WorthQuerySelectedProductOperation<'runtime, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    installed: &WorthQueryInstalledWorkflowReadBinding<'_, Schema, Spec, B>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    input: B::Input,
    request: &WorthQueryRequestScope,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
) -> Result<Option<AdmittedOriginalWorkflowCandidate<'data, 'runtime, Schema, B>>, Denial>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    B: ApplicationWorkflowReadBinding<Schema>,
{
    if installed.route() != ApplicationWorkflowReadRoute::Subject {
        return Err(Denial::PolicyMismatch);
    }
    let access = admit(current, phase, installed, principal, input, request)?;
    // Caller supplied domain input, not a principal/entity forged from the missing target.
    if access.resource_entity_id() != original.subject_entity_id()
        || !std::ptr::eq(current.application(), data.application())
        || current.product().branch_identity() != data.product().branch_identity()
    {
        return Err(Denial::PolicyMismatch);
    }
    let execution = phase
        .execution_request_for(&current.application().product_runtime)
        .map_err(Denial::Phase)?;
    let graph = current
        .application()
        .runtime
        .primary_graph()
        .ok_or(Denial::BasisUnavailable)?;
    let snapshot = data.application_basis().snapshot_handle();
    // Capability custody owns a native snapshot whose release reacquires this runtime.
    // Keep it outside the locked callback on missing, foreign-Spec and refusal paths.
    let candidate = graph.integration_handle().with_runtime(&execution,|runtime| {
        let layout=graph.layout().workflow();
        ensure_selected_field_indexes_admitted(&execution, runtime, data.product().relational_basis(), [Some(layout.instance.identity_index_id),None]).map_err(|stop| match stop {
                SelectedFieldIndexAdmissionStop::Admission(cause)=>Denial::Preparation(cause),
                SelectedFieldIndexAdmissionStop::Native=>Denial::SelectedIndexUnavailable,
            })?;
        // The authentic canonical start owner fixes text at64 bytes. Pay actual text before copying.
        let text=original.lookup_identity();

        let value=AspectValue::String(InternedString::Raw(text.to_owned()));
        let view=runtime.read_truth().project_snapshot(snapshot).ok_or(Denial::BasisUnavailable)?;
        let selected=runtime.index_access().execute_bounded_entity_field_lookup(
            BoundedEntityFieldSelectedLookup::new(&view,layout.instance.identity_index_id,layout.instance.entity_kind,
                &layout.instance.identity,&value,2),
            BoundedIndexParityMode::Production,&execution).map_err(|cause| Denial::Index(cause.kind()))?;
        if selected.overflowed() || selected.candidate_entity_ids().len()>1 { return Err(Denial::Corrupt); }
        let Some(instance)=selected.candidate_entity_ids().first().copied() else { return Ok(None); };
        for (field,expected) in [(&layout.instance.protocol_version,crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION),
            (&layout.instance.subject_partition,u64::from(original.subject_entity_id().partition_value())),
            (&layout.instance.subject_slot,original.subject_entity_id().local_slot_value()),
            (&layout.instance.subject_generation,u64::from(original.subject_entity_id().generation_value()))] {
            let matches=observe_field_value_borrowed(runtime,&execution,snapshot,instance,layout.instance.entity_kind,field,
                |value| value==Some(&AspectValue::UInt64(expected))).map_err(Denial::Relational)?;
            if matches!=Some(true) { return Err(Denial::Corrupt); }
        }
        let lineages=observe_adjacency(runtime,&execution,snapshot,layout.instance_lineage_relation,instance,
            WorthQueryApplicationAdjacencyDirection::Outgoing,2).map_err(Denial::Execution)?.ok_or(Denial::BasisUnavailable)?;
        let [lineage]=lineages.as_slice() else { return Err(Denial::Corrupt); };
        let pinned=runtime.read_truth().pin_read_capability(snapshot,&[],&execution).map_err(Denial::PinnedRead)?;
        let matches=super::text::inspect(runtime,&execution,&pinned,lineage.to,layout.lineage.entity_kind,&layout.lineage.spec,
            |value|value.filter(|value|!value.is_empty()).map(|value|value==Spec::IDENTITY.as_str()))?;
        match matches {
            Some(true)=>{},
            Some(false)=>return Ok(None),
            None=>return Err(Denial::Corrupt),
        }
        Ok(Some(OriginalWorkflowCandidate { data,instance,lineage:lineage.to,subject:original.subject_entity_id() }))
    })?;
    Ok(candidate.map(|candidate| (candidate, access)))
}
