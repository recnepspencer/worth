//! Exact, principal-first definition metadata at the caller's retained observation.
use super::{
    admission::{admit, WorthQueryWorkflowReadDenial as Denial},
    *,
};
use crate::domain_computation::primary_graph::application_installation::WorthQueryWorkflowVocabulary;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQuerySelectedProductOperation,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDefinitionReadTarget, ApplicationWorkflowReadBinding,
    ApplicationWorkflowReadRoute, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightBudget;

/// Exact metadata result owns its authentic data basis, never read permission.
pub struct WorthQueryWorkflowDefinitionRead {
    basis: super::WorthQueryWorkflowReadBasis,
    definition: WorthQueryWorkflowDefinitionDiscovery,
}
impl WorthQueryWorkflowDefinitionRead {
    pub fn basis(&self) -> &super::WorthQueryWorkflowReadBasis {
        &self.basis
    }
    pub fn definition(&self) -> &WorthQueryWorkflowDefinitionDiscovery {
        &self.definition
    }
}
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Fresh metadata-only disclosure occurs before lookup; no instance permission is borrowed.
    /// Execution custody and a name alone cannot use the former ungated call shape.
    /// ```compile_fail,E0061
    /// use worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation;
    /// use worth_query_installation::facade::ApplicationSchema;
    /// use worth_query_declaration::facade::application_program::{ApplicationWorkflowSpec,ApplicationWorkflowReadBinding,ApplicationWorkflowDefinitionReadTarget};
    /// fn no_ungated<S,W,B>(selected:&WorthQuerySelectedProductOperation<'_,S>,execution:&worth_execution::ExecutionRequest<'_, '_>,target:&ApplicationWorkflowDefinitionReadTarget<W>)
    /// where S:ApplicationSchema,W:ApplicationWorkflowSpec<Schema=S>,B:ApplicationWorkflowReadBinding<S> {
    ///     selected.discover_workflow_definition::<W,B>(execution,target);
    /// }
    /// ```
    pub fn discover_workflow_definition<Spec, B>(
        &self,
        data: &WorthQuerySelectedProductOperation<'_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        vocabulary: &WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        input: B::Input,
        request: &WorthQueryRequestScope,
        target: &ApplicationWorkflowDefinitionReadTarget<Spec>,
    ) -> Result<WorthQueryWorkflowDefinitionRead, Denial>
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
        let binding = workflow
            .resolve_read_binding::<B>(
                self.application().installed_schema(),
                ApplicationWorkflowReadRoute::DefinitionMetadata,
            )
            .map_err(Denial::Installation)?;
        let profile = self
            .application()
            .runtime
            .application_query_resource_profile();
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: profile.maximum_work().map(|work| work.get() as u64),
            maximum_preparation_bytes: profile
                .maximum_inline_index_bytes()
                .map(|bytes| bytes.get() as u64),
        });
        let _access = admit(
            self,
            phase,
            &binding,
            principal,
            input,
            request,
            &mut admission,
        )?;
        let execution = phase
            .execution_request_for(&self.application().product_runtime)
            .map_err(Denial::Phase)?;
        let graph = self
            .application()
            .runtime
            .primary_graph()
            .ok_or(Denial::BasisUnavailable)?;
        let result = graph
            .integration_handle()
            .with_runtime(&execution, |runtime| {
                let layout = graph.layout().workflow();
                let snapshot = data.application_basis().snapshot_handle();
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
                    return Ok(WorthQueryWorkflowDefinitionDiscovery::Unpublished);
                };
                let pinned = runtime
                    .read_truth()
                    .pin_read_capability(snapshot, &[], &execution)
                    .map_err(Denial::PinnedRead)?;
                let links = observe_adjacency(
                    runtime,
                    &execution,
                    snapshot,
                    layout.current_definition_relation,
                    lineage,
                    WorthQueryApplicationAdjacencyDirection::Outgoing,
                    2,
                )
                .map_err(Denial::Execution)?
                .ok_or(Denial::BasisUnavailable)?;
                let definition = match links.as_slice() {
                    [] => WorthQueryWorkflowDefinitionDiscovery::Retired,
                    [link] => {
                        let content = super::text::inspect(
                            runtime,
                            &execution,
                            &pinned,
                            link.to,
                            layout.definition.entity_kind,
                            &layout.definition.content_identity,
                            |text| {
                                text.and_then(
                                    ApplicationWorkflowDefinitionContentIdentity::from_recorded,
                                )
                            },
                        )?
                        .ok_or(Denial::Corrupt)?;
                        WorthQueryWorkflowDefinitionDiscovery::Current(
                            PublishedWorkflowDefinitionRef::retained(
                                data.product().product_branch(),
                                link.to,
                                content,
                            ),
                        )
                    }
                    _ => return Err(Denial::Corrupt),
                };
                Ok(definition)
            })?;
        Ok(WorthQueryWorkflowDefinitionRead {
            basis: super::WorthQueryWorkflowReadBasis::retained(data),
            definition: result,
        })
    }
}
