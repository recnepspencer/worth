//! Fresh disclosure and selected native ordered pages at the retained data basis.
use super::{admission::WorthQueryWorkflowReadDenial as Denial, *};
use crate::domain_computation::primary_graph::{
    application_attempt::workflow_instance_observation::{
        history_projection, read_fields::ReadFields,
    },
    application_installation::WorthQueryWorkflowVocabulary,
    output_lineage::invalidation::InvalidationEditAdmission,
    WorthQueryAdvancementPhase, WorthQuerySelectedProductOperation,
};
use worth_execution::ExecutionArrayBuilder;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowReadBinding, ApplicationWorkflowReadRoute, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    indexes::{
        BoundedRelatedEntityOrderedLookupRequest, BoundedRelatedEntityPrefixLookupRequest,
        MAX_BOUNDED_RELATED_ENTITY_PAGE_WIDTH,
    },
    mvcc::CompanionPreflightBudget,
};
impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    /// Reads only the native ordered page; never reconstructs and slices whole history.
    pub fn read_workflow_history<W, Subject, Audit>(
        &self,
        data: &WorthQuerySelectedProductOperation<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        vocabulary: &WorthQueryWorkflowVocabulary<'_, S, W>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        input: Subject::Input,
        audit_input: Audit::Input,
        request: &WorthQueryRequestScope,
        target: &WorthQueryWorkflowReadTarget,
        width: usize,
        indexes: WorthQueryWorkflowIndexRead,
        after: Option<&WorthQueryWorkflowHistoryPage>,
    ) -> Result<WorthQueryWorkflowHistoryPage, Denial>
    where
        W: ApplicationWorkflowSpec<Schema = S>,
        Subject: ApplicationWorkflowReadBinding<S>,
        Audit: ApplicationWorkflowReadBinding<S>,
    {
        if !std::ptr::eq(vocabulary.runtime(), self.application()) {
            return Err(Denial::PolicyMismatch);
        }
        let workflow = vocabulary.workflow_spec_for(self);
        let subject = workflow
            .resolve_read_binding::<Subject>(
                self.application().installed_schema(),
                ApplicationWorkflowReadRoute::Subject,
            )
            .map_err(Denial::Installation)?;
        let profile = self
            .application()
            .runtime
            .application_query_resource_profile();
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: profile.maximum_work().map(|work| work.get() as u64),
            maximum_preparation_bytes: profile.maximum_inline_index_bytes().get() as u64,
        });
        let (candidate, access) = target.admit(
            self,
            data,
            phase,
            &subject,
            principal,
            input,
            request,
            &mut admission,
        )?;
        let audited = if workflow
            .observation_policy()
            .rule(ApplicationWorkflowReadRoute::Audit)
            .is_some()
        {
            let audit_binding = workflow
                .resolve_read_binding::<Audit>(
                    self.application().installed_schema(),
                    ApplicationWorkflowReadRoute::Audit,
                )
                .map_err(Denial::Installation)?;
            audit::admit_audit(
                self,
                &candidate,
                phase,
                &subject,
                &access,
                &audit_binding,
                principal,
                audit_input,
                request,
                &mut admission,
            )?
            .is_some()
        } else {
            false
        };
        if width == 0
            || width > MAX_BOUNDED_RELATED_ENTITY_PAGE_WIDTH
            || width
                > workflow
                    .resources()
                    .maximum_retained_transitions_per_instance() as usize
        {
            return Err(Denial::PolicyMismatch);
        }
        if let Some(prior) = after
            && (!prior
                .basis
                .0
                .belongs_to_selected_occurrence(self.application(), data.product().observation())
                || prior.instance != target.instance
                || prior.subject != target.subject
                || prior.scope != access.resource_entity_id()
                || prior.width != width
                || &prior.support != workflow.support_identity_bytes()
                || prior.basis.selected_commit() != data.product().selected_commit()
                || prior.basis.branch_identity() != data.product().branch_identity()
                || prior.continuation().is_none())
        {
            return Err(Denial::ContinuationMismatch);
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
            .transitions;
        let (rows, native) = graph
            .integration_handle()
            .with_runtime(&execution, |runtime| {
                let snapshot = data.application_basis().snapshot_handle();
                let layout = graph.layout().workflow();
                enumeration::currency::prepare(
                    runtime,
                    &execution,
                    data.product().relational_basis(),
                    index,
                    indexes,
                    &mut admission,
                )?;
                let pinned = runtime
                    .read_truth()
                    .pin_read_capability(snapshot, &[index], &execution)
                    .map_err(Denial::PinnedRead)?;
                let ordered = BoundedRelatedEntityOrderedLookupRequest::new(
                    snapshot.clone(),
                    index,
                    target.instance,
                    layout.transition.entity_kind,
                    None,
                    width,
                )
                .map_err(Denial::PageShape)?;
                let native = pinned
                    .execute_bounded_related_entity_prefix_lookup(
                        BoundedRelatedEntityPrefixLookupRequest::new(
                            ordered,
                            &[],
                            after.and_then(|p| p.native.next_boundary()),
                        ),
                        &execution,
                    )
                    .map_err(Denial::Page)?;
                let fields = ReadFields::new(runtime, &execution, snapshot)?;
                let definition = fields
                    .one(layout.instance_definition_relation, target.instance)?
                    .ok_or(Denial::Corrupt)?;
                let mut rows = ExecutionArrayBuilder::allocate(
                    native.child_entity_ids().len(),
                    execution.allocation_policy(),
                )
                .map_err(Denial::Allocation)?;
                let mut occurrence = after
                    .and_then(|p| p.entries().last())
                    .map(|r| r.occurrence());
                for transition in native.child_entity_ids() {
                    let row = history_projection::observe(
                        &fields,
                        layout,
                        data.product().product_branch(),
                        target.instance,
                        definition,
                        *transition,
                        audited,
                        usize::from(workflow.resources().maximum_definition_connections()),
                    )?;
                    if occurrence.is_some_and(|prior| prior >= row.occurrence()) {
                        return Err(Denial::Corrupt);
                    }
                    occurrence = Some(row.occurrence());
                    rows.push(row).map_err(Denial::Allocation)?;
                }
                Ok((rows.seal().map_err(Denial::Allocation)?, native))
            })?;
        Ok(WorthQueryWorkflowHistoryPage {
            basis: WorthQueryWorkflowReadBasis::retained(data),
            instance: target.instance,
            subject: target.subject,
            scope: access.resource_entity_id(),
            support: *workflow.support_identity_bytes(),
            width,
            rows,
            native,
        })
    }
}
