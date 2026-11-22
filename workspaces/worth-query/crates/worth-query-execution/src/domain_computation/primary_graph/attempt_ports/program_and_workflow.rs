//! `SupportedProgramRecordPort` and `WorkflowVocabularyPort` impls for L6 types.

use std::any::{Any, TypeId};

use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventSigningOwner;
use worth_query_declaration::facade::{
    application_program::{ApplicationProgramRevision, ApplicationWorkflowSpec},
    application_schema::ApplicationSchema,
};
use worth_query_execution_attempt::facade::ports::{
    SupportedProgramRecordPort, WorkflowVocabularyPort,
};
use worth_query_installation::facade::{
    WorthQueryInstalledActionInventory, WorthQueryInstalledApplicationWorkflowSpec,
};

use crate::domain_computation::primary_graph::application_installation::{
    WorthQuerySupportedProgramRecord, WorthQueryWorkflowVocabulary,
};
use crate::equality_type_id::EqualityTypeId;

pub fn downcast_supported_program(
    port: &dyn SupportedProgramRecordPort,
) -> &WorthQuerySupportedProgramRecord {
    port.as_any()
        .downcast_ref::<WorthQuerySupportedProgramRecord>()
        .expect("program support carries WorthQuerySupportedProgramRecord")
}

impl SupportedProgramRecordPort for WorthQuerySupportedProgramRecord {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn action_inventory(&self) -> WorthQueryInstalledActionInventory<'_> {
        self.installed.action_inventory()
    }

    fn revision(&self) -> &ApplicationProgramRevision {
        self.revision()
    }

    fn declares_binding(&self, binding: TypeId) -> bool {
        self.declares_action(EqualityTypeId::from(binding))
    }
}

impl<Schema, Spec> WorkflowVocabularyPort<Schema, Spec>
    for WorthQueryWorkflowVocabulary<'_, Schema, Spec>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    fn workflow_spec_at_revision(
        &self,
        revision: &ApplicationProgramRevision,
    ) -> Option<&WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>> {
        self.workflow_spec_at_revision(revision)
    }

    fn authentication_owner(&self) -> &WorthQueryAuthenticationEventSigningOwner<Schema> {
        self.authentication_owner()
    }
}
