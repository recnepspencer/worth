//! One current-security / exact-retained-data native governed read, with no lasting permit.
use super::admission::WorthQueryWorkflowReadDenial as Denial;
use super::audit;
use super::original_intent;
use super::view::*;
use crate::domain_computation::primary_graph::application_attempt::workflow_instance_observation::governed_record;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryWorkflowInstanceDiscoveryIntent;
use crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary;

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadBinding;
use worth_query_declaration::facade::application_program::ApplicationWorkflowReadRoute;
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Authenticates fresh current read authority before inspecting the original target.
    /// `data` must be an explicitly selected retained observation of this same branch.
    /// The returned view grants no command right; only a genuinely live target yields a locator.
    pub fn read_original_workflow<Spec, Subject, Audit>(
        &self,
        data: &WorthQuerySelectedProductOperation<'_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        vocabulary: &WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        input: Subject::Input,
        audit_input: Audit::Input,
        request: &WorthQueryRequestScope,
        original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
    ) -> Result<WorthQueryWorkflowInstanceRead, Denial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Subject: ApplicationWorkflowReadBinding<Schema>,
        Audit: ApplicationWorkflowReadBinding<Schema>,
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
        let audit_binding = if workflow
            .observation_policy()
            .rule(ApplicationWorkflowReadRoute::Audit)
            .is_some()
        {
            Some(
                workflow
                    .resolve_read_binding::<Audit>(
                        self.application().installed_schema(),
                        ApplicationWorkflowReadRoute::Audit,
                    )
                    .map_err(Denial::Installation)?,
            )
        } else {
            None
        };

        let Some((target, access)) = original_intent::lookup(
            self, data, phase, &subject, principal, input, request, original,
        )?
        else {
            return Ok(WorthQueryWorkflowInstanceRead::Missing {
                basis: WorthQueryWorkflowReadBasis::retained(data),
            });
        };
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(Denial::Phase)?;
        let graph = self
            .application()
            .runtime
            .primary_graph()
            .ok_or(Denial::BasisUnavailable)?;
        let current_basis = self.product().selected_commit() == data.product().selected_commit();
        let record = graph
            .integration_handle()
            .with_runtime(&execution, |runtime| {
                governed_record::observe(
                    &execution,
                    runtime,
                    data.application_basis().snapshot_handle(),
                    graph.layout().workflow(),
                    data.product().product_branch(),
                    target.instance,
                    current_basis,
                    target.lineage,
                    workflow.program_revision(),
                    self.application()
                        .installed_program_support()
                        .ok_or(Denial::PolicyMismatch)?,
                    workflow.resources(),
                )
            })?;
        if &record.original_start_revision != original.original_start_revision() {
            return Err(Denial::Corrupt);
        }
        let audit = audit::observe(
            self,
            &target,
            phase,
            &subject,
            &access,
            audit_binding.as_ref(),
            principal,
            audit_input,
            request,
            &record.history,
        )?;
        Ok(project_record(record, data, audit))
    }
}

pub(super) fn project_record<Schema: ApplicationSchema>(
    mut record: governed_record::WorkflowRecord,
    data: &WorthQuerySelectedProductOperation<'_, Schema>,
    audit: WorthQueryWorkflowAudit,
) -> WorthQueryWorkflowInstanceRead {
    let posture = if record.occurrence != data.product().product_branch().occurrence_ordinal() {
        WorthQueryWorkflowHistoricalPosture::Fork
    } else if record.state != WorthQueryWorkflowObservedState::Ready {
        WorthQueryWorkflowHistoricalPosture::Ended
    } else if !record.live {
        WorthQueryWorkflowHistoricalPosture::Unlinked
    } else {
        WorthQueryWorkflowHistoricalPosture::RetainedBasis
    };
    let live = record.reference.take();
    let target = super::target::WorthQueryWorkflowReadTarget::observed(
        data,
        record.instance,
        record.lineage,
        record.subject,
        record.dependencies,
    );
    let observation = WorthQueryWorkflowObservation {
        target,
        basis: WorthQueryWorkflowReadBasis::retained(data),
        state: record.state,
        history: record.history,
        audit,
    };
    match live {
        Some(instance) => WorthQueryWorkflowInstanceRead::Live {
            instance,
            observation,
        },
        None => WorthQueryWorkflowInstanceRead::Historical {
            posture,
            observation,
        },
    }
}
