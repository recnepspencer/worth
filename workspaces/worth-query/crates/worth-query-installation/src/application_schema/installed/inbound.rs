use worth_query_declaration::facade::application_program::ApplicationWorkflowInboundRef;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use super::WorthQueryInstalledApplicationSchema;

impl<Schema> WorthQueryInstalledApplicationSchema<Schema> {
    pub(crate) fn installed_workflow_inbound_ref(
        &self,
        operation: &str,
    ) -> Option<ApplicationWorkflowInboundRef> {
        let mut members = self.schema.members().iter().filter_map(|member| {
            let ApplicationSchemaMember::OperationInboundOccurrence {
                operation: declared,
                effect,
                protocol,
                source_identity,
                limits,
            } = member
            else {
                return None;
            };
            (declared == operation).then_some((effect, protocol, source_identity, limits))
        });
        let (effect, protocol, source, limits) = members.next()?;
        if members.next().is_some() {
            return None;
        }
        self.member_provenance.workflow_inbound_ref(
            effect,
            protocol.clone(),
            source.clone(),
            *limits,
        )
    }
}
