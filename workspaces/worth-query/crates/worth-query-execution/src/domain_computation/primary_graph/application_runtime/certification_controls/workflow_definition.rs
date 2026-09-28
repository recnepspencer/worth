//! Read-only probe of a published definition's revision provenance.

use worth_foundational::facade::AspectValue;
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_attempt::observe_field_value;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// The revision `definition` was published under and the latest revision
    /// program adoption carried it to, as its branch's owner truth reads now.
    #[doc(hidden)]
    pub fn workflow_definition_revisions_for_test(
        &self,
        definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef,
    ) -> (Option<AspectValue>, Option<AspectValue>) {
        let selected = self
            .on_branch(definition.branch())
            .select()
            .expect("the definition branch remains admitted");
        let observation = selected.product().relational_basis().observation();
        self.runtime
            .primary_graph()
            .expect("the application retains its primary graph")
            .integration_handle()
            .with_query_runtime_mut(|runtime, layout| {
                let snapshot = runtime
                    .snapshots()
                    .snapshot_for_observation(&observation)
                    .expect("the selected basis admits one probe snapshot");
                let definition_layout = &layout.workflow().definition;
                let read = |locator| {
                    observe_field_value(
                        runtime,
                        &snapshot,
                        definition.entity_id(),
                        definition_layout.entity_kind,
                        locator,
                    )
                };
                let revisions = (
                    read(&definition_layout.program_revision),
                    read(&definition_layout.carried_revision),
                );
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                revisions
            })
    }
}
