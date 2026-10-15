//! Definition-selected governed pages over native retained prefix generations.
use super::{
    admission::{admit, WorthQueryWorkflowReadDenial as Denial},
    read::project_record,
    *,
};
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::governed_record,
    application_installation::WorthQueryWorkflowVocabulary,
    output_lineage::invalidation::InvalidationEditAdmission, WorthQueryAdvancementPhase,
    WorthQuerySelectedProductOperation,
};
use std::marker::PhantomData;
use worth_execution::{ExecutionArray, ExecutionArrayBuilder};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDefinitionReadTarget, ApplicationWorkflowReadBinding,
    ApplicationWorkflowReadRoute, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    identity::EntityId,
    indexes::{
        BoundedRelatedEntityOrderedLookupRequest, BoundedRelatedEntityPrefixLookupRequest,
        BoundedRelatedEntityPrefixPage, DerivedIndexMaintenanceBudget,
    },
    mvcc::CompanionPreflightBudget,
};
mod currency;
/// Explicit selected scope; omitting a subject never widens a subject rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowReadExtent {
    Subject,
    Definition,
}
/// Hot reads never reconstruct. Cold work requires explicit finite native controls.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryWorkflowIndexRead {
    PublishedOnly,
    Reconstruct(DerivedIndexMaintenanceBudget),
}
/// Complete funded descriptive page; also owns its opaque exact-basis continuation.
/// It is not a complete program/adoption safety inventory and carries no permission.
pub struct WorthQueryWorkflowInstancePage<'target, Schema, Spec, B>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    B: ApplicationWorkflowReadBinding<Schema>,
{
    basis: WorthQueryWorkflowReadBasis,
    target: &'target ApplicationWorkflowDefinitionReadTarget<Spec>,
    extent: WorthQueryWorkflowReadExtent,
    subject: Option<EntityId>,
    resource_scope: EntityId,
    width: usize,
    rows: ExecutionArray<WorthQueryWorkflowInstanceRead>,
    native: Option<BoundedRelatedEntityPrefixPage>,
    marker: PhantomData<fn() -> (Schema, B)>,
}
impl<
        S: ApplicationSchema,
        W: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationWorkflowReadBinding<S>,
    > WorthQueryWorkflowInstancePage<'_, S, W, B>
{
    pub fn basis(&self) -> &WorthQueryWorkflowReadBasis {
        &self.basis
    }
    pub fn entries(&self) -> &[WorthQueryWorkflowInstanceRead] {
        self.rows.elements()
    }
    /// Retain this opaque owner for continuation; fresh disclosure is required on every call.
    pub fn continuation(&self) -> Option<&Self> {
        self.native.as_ref()?.next_boundary().map(|_| self)
    }
}
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Enumerates exactly the selected definition and declared subject/definition extent.
    /// Base pages explicitly withhold actors; separate audit admission never derives from a page.
    pub fn read_workflow_instances<'target, Spec, B>(
        &self,
        data: &WorthQuerySelectedProductOperation<'_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        vocabulary: &WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        input: B::Input,
        request: &WorthQueryRequestScope,
        target: &'target ApplicationWorkflowDefinitionReadTarget<Spec>,
        extent: WorthQueryWorkflowReadExtent,
        width: usize,
        index_read: WorthQueryWorkflowIndexRead,
        after: Option<&WorthQueryWorkflowInstancePage<'_, Schema, Spec, B>>,
    ) -> Result<WorthQueryWorkflowInstancePage<'target, Schema, Spec, B>, Denial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        B: ApplicationWorkflowReadBinding<Schema>,
    {
        if !std::ptr::eq(vocabulary.runtime(), self.application())
            || !std::ptr::eq(self.application(), data.application())
            || self.product().branch_identity() != data.product().branch_identity()
        {
            return Err(Denial::PolicyMismatch);
        }
        let workflow = vocabulary.workflow_spec_for(self);
        let route = match extent {
            WorthQueryWorkflowReadExtent::Subject => ApplicationWorkflowReadRoute::Subject,
            WorthQueryWorkflowReadExtent::Definition => ApplicationWorkflowReadRoute::Definition,
        };
        let binding = workflow
            .resolve_read_binding::<B>(self.application().installed_schema(), route)
            .map_err(Denial::Installation)?;
        let profile = self
            .application()
            .runtime
            .application_query_resource_profile();
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: profile.maximum_work().get() as u64,
            maximum_preparation_bytes: profile.maximum_inline_index_bytes().get() as u64,
        });
        let access = admit(
            self,
            phase,
            &binding,
            principal,
            input,
            request,
            &mut admission,
        )?;
        let resource_scope = access.resource_entity_id();
        let subject = if extent == WorthQueryWorkflowReadExtent::Subject {
            Some(access.resource_entity_id())
        } else {
            None
        };
        if let Some(prior) = after {
            if prior.target.identity() != target.identity()
                || prior.extent != extent
                || prior.subject != subject
                || prior.resource_scope != resource_scope
                || prior.width != width
                || prior.basis.selected_commit() != data.product().selected_commit()
                || prior.basis.branch_identity() != data.product().branch_identity()
                || prior.continuation().is_none()
            {
                return Err(Denial::ContinuationMismatch);
            }
        }
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(Denial::Phase)?;
        let graph = self
            .application()
            .runtime
            .primary_graph()
            .ok_or(Denial::BasisUnavailable)?;
        let index = graph
            .layout()
            .workflow_reads()
            .ok_or(Denial::SelectedIndexUnavailable)?
            .instances;
        // Validate native width even for absent targets, after the uniform principal gate.
        if width == 0
            || width > worth_relational::facade::indexes::MAX_BOUNDED_RELATED_ENTITY_PAGE_WIDTH
        {
            return Err(Denial::PolicyMismatch);
        }
        let (rows, native) = graph
            .integration_handle()
            .with_runtime(&execution, |runtime| {
                let snapshot = data.application_basis().snapshot_handle();
                let layout = graph.layout().workflow();
                let Some(lineage) = super::selection::lineage::<Spec>(
                    runtime,
                    &execution,
                    snapshot,
                    data.product().relational_basis(),
                    layout,
                    target.identity().as_str(),
                    &mut admission,
                )?
                else {
                    let rows = ExecutionArrayBuilder::allocate(0, execution.allocation_policy())
                        .map_err(Denial::Allocation)?
                        .seal()
                        .map_err(Denial::Allocation)?;
                    return Ok((rows, None));
                };
                currency::prepare(
                    runtime,
                    &execution,
                    data.product().relational_basis(),
                    index,
                    index_read,
                    &mut admission,
                )?;
                let pinned = runtime
                    .read_truth()
                    .pin_read_capability(snapshot, &[index], &execution)
                    .map_err(Denial::PinnedRead)?;
                let values = subject.map(|subject| {
                    [
                        AspectValue::UInt64(subject.partition_value_u64()),
                        AspectValue::UInt64(subject.local_slot_value()),
                        AspectValue::UInt64(u64::from(subject.generation_value())),
                    ]
                });
                let prefix = values.as_ref().map_or(&[][..], |values| &values[..]);
                let ordered = BoundedRelatedEntityOrderedLookupRequest::new(
                    snapshot.clone(),
                    index,
                    lineage,
                    layout.instance.entity_kind,
                    None,
                    width,
                )
                .map_err(Denial::PageShape)?;
                let native = pinned
                    .execute_bounded_related_entity_prefix_lookup(
                        BoundedRelatedEntityPrefixLookupRequest::new(
                            ordered,
                            prefix,
                            after
                                .and_then(|page| page.native.as_ref())
                                .and_then(|page| page.next_boundary()),
                        ),
                        &execution,
                    )
                    .map_err(Denial::Page)?;
                let mut rows = ExecutionArrayBuilder::allocate(
                    native.child_entity_ids().len(),
                    execution.allocation_policy(),
                )
                .map_err(Denial::Allocation)?;
                for instance in native.child_entity_ids() {
                    // The native index verifies child kind, relation and complete ordering values.
                    // The canonical record validates current protocol/start provenance at this SAME basis.
                    let record = governed_record::observe(
                        &execution,
                        runtime,
                        snapshot,
                        layout,
                        data.product().product_branch(),
                        *instance,
                        self.product().selected_commit() == data.product().selected_commit(),
                        lineage,
                        workflow.program_revision(),
                        self.application()
                            .installed_program_support()
                            .ok_or(Denial::PolicyMismatch)?,
                        workflow.resources(),
                    )?;
                    rows.push(project_record(
                        record,
                        data,
                        WorthQueryWorkflowAudit::Withheld,
                    ))
                    .map_err(Denial::Allocation)?;
                }
                Ok((rows.seal().map_err(Denial::Allocation)?, Some(native)))
            })?;
        Ok(WorthQueryWorkflowInstancePage {
            basis: WorthQueryWorkflowReadBasis::retained(data.retain_application_read()),
            target,
            extent,
            subject,
            resource_scope,
            width,
            rows,
            native,
            marker: PhantomData,
        })
    }
}
