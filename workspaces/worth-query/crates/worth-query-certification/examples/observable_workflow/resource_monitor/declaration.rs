//! This contract admits recovery ownership; the court never executes a corrector.
use crate::document_retention_model::{
    retention_entry::{DocumentRetentionWrittenBinding, SetRetentionDenialBinding},
    schema::*,
};
use worth_query_decl::facade::{
    application_aftermath::*, application_operation::*, application_schema::*,
    worth_query_application_contribution, worth_query_capability, worth_query_mutation_binding,
    worth_query_operation, worth_query_operation_emits, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct MonitorInput {
    pub identity: String,
    pub retention_days: u64,
}
worth_query_structured_value_binding!(pub MonitorInputBinding for MonitorInput { identity: "neutral.recoverable-archive.input.v1" });
worth_query_operation!(pub MonitorOperation for DocumentRetentionSchema, input MonitorInputBinding);
worth_query_operation_reads!(MonitorOperation => [DocumentIdentityField, DocumentRetentionField]);
worth_query_operation_writes!(MonitorOperation => [DocumentRetentionField]);
worth_query_operation_emits!(MonitorOperation => [crate::effect::ArchiveEffect]);
worth_query_capability!(pub MonitorInspectionCapability in DocumentRetentionSchema);
impl
    worth_query_decl::facade::application_capability::ApplicationCapabilityRequest<
        DocumentRetentionSchema,
        MonitorInspectionCapability,
    > for MonitorInput
{
    type Scope = Document;
    type Context = crate::document_retention_model::workflow::governed_read::WorkflowReadContext;

    fn capability_request(
        &self,
    ) -> Result<
        worth_query_decl::facade::application_capability::ApplicationCapabilityRequestProjection<
            DocumentRetentionSchema,
            Document,
            Self::Context,
        >,
        worth_query_decl::facade::application_capability::ApplicationCapabilityRequestProjectionDenial,
    >{
        use crate::document_retention_model::workflow::governed_read::WorkflowReadContext;
        use worth_query_decl::facade::application_capability::*;
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::reference(),
                ApplicationEncodedScalarValue::try_new(self.identity.clone()).unwrap(),
            ),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "read-workflow-subject".to_owned(),
            )
            .unwrap(),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "reviewed-document".to_owned(),
            )
            .unwrap(),
            ApplicationCapabilityRequestContext::new(WorkflowReadContext::reference()),
        ))
    }
}
fn identity(input: &MonitorInput) -> String {
    input.identity.clone()
}
worth_query_mutation_binding!(
    pub MonitorBinding for MonitorInput, schema DocumentRetentionSchema,
    identity "neutral.recoverable-archive.binding.v1", input MonitorInputBinding,
    operation MonitorOperation, result DocumentRetentionWrittenBinding,
    idempotency u64, identity "neutral.recoverable-archive.key.v1",
    decision WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>, denial SetRetentionDenialBinding,
    handler identity "neutral.recoverable-archive.handler.v1",
    admission ApplicationCapabilityAdmission<MonitorInspectionCapability>, program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal DocumentPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Document, DocumentFacts, DocumentIdentityField, String, ReadOnly, NoApplicationUnit,
    field DocumentIdentityField::reference(), value identity,
);
impl ApplicationCapabilityMutationBinding<DocumentRetentionSchema> for MonitorBinding {
    type Capability = MonitorInspectionCapability;
}

fn aftermath() -> DeclaredApplicationAftermathContract<DocumentRetentionSchema> {
    DeclaredApplicationAftermathContract::runtime_with_external_owner(
        DeclaredCorrectionMechanism::RecordedInverse(
            DeclaredRecordedInverse::new(
                "neutral.recoverable-archive.retention-preimage.v1",
                DeclaredLoweringCorrespondenceRef::new(
                    "neutral.recoverable-archive.retention-lowering.v1",
                )
                .unwrap(),
                DeclaredAftermathPostcondition::ExactPriorTruth,
                DeclaredPreImageDemand::new(
                    [DeclaredPreImageLocus::from_field(
                        DocumentRetentionField::reference(),
                    )],
                    512,
                )
                .unwrap(),
            )
            .unwrap(),
        ),
        DeclaredReconciliationProcedure::new("neutral.archive-owner-confirms-original-effect")
            .unwrap(),
    )
}
worth_query_application_contribution! {
    pub contribution MonitorContribution in DocumentRetentionSchema {
        identity: "neutral.recoverable-archive.application.v1",
        members: |schema| { schema
            .operation(MonitorOperation::reference().definition()
                .external_effect(crate::effect::ArchiveEffect::reference(), WorthQueryExternalEffectCorrelationFamily::new("neutral-recoverable-archive").unwrap())
                .aftermath(aftermath()).finish())
            .operation_projection_work_budget(MonitorOperation::reference(), 32)
            .operation_read_field(MonitorOperation::reference(), DocumentIdentityField::reference())
            .operation_read_field(MonitorOperation::reference(), DocumentRetentionField::reference())
            .operation_write(MonitorOperation::reference(), DocumentRetentionField::reference())
            .operation_emit(MonitorOperation::reference(), crate::effect::ArchiveEffect::reference())
            .capability(crate::document_retention_model::workflow::governed_read::subject_contract(MonitorInspectionCapability::reference(), MonitorOperation::reference()))
            .application_mutation_binding::<MonitorBinding>()
            .application_resource(super::resource::definition()) }
    }
}
impl WorthQueryApplicationContribution<DocumentRetentionSchema> for MonitorContribution {
    type Configuration = super::Configuration;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DocumentRetentionSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<MonitorBinding, _>(super::handler::Handler)?;
        setup.resource::<super::resource::Monitor, _, super::producer::Clock, _>(
            super::producer::Producer(configuration.state),
            configuration.clock,
        )
    }
}
