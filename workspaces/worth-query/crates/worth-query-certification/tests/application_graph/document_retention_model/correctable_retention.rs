//! A guarded source operation with a separately admitted ordinary inverse.
use super::{retention_entry::*, schema::*};
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    *,
};

#[path = "correctable_retention/correction.rs"]
mod correction;
pub use correction::RestoreReviewedRetention;

worth_query_operation!(pub CorrectableRetention for DocumentRetentionSchema, input SetRetentionInputBinding; access {
    read_fields: [DocumentIdentityField, DocumentRetentionField];
    writes: [DocumentRetentionField];
});

pub struct CorrectableRetentionBinding;
#[derive(Clone)]
pub struct CorrectableRetentionIntent {
    pub input: SetRetentionInput,
}
type Reviewed = ReviewedSetRetentionBinding;
type Schema = DocumentRetentionSchema;

impl ApplicationMutationBinding<Schema> for CorrectableRetentionBinding {
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = <Reviewed as ApplicationMutationBinding<Schema>>::Result;
    type ResultBinding = <Reviewed as ApplicationMutationBinding<Schema>>::ResultBinding;
    type IdempotencyKey = u64;
    type Operation = CorrectableRetention;
    type Decision = <Reviewed as ApplicationMutationBinding<Schema>>::Decision;
    type Denial = <Reviewed as ApplicationMutationBinding<Schema>>::Denial;
    type DenialBinding = <Reviewed as ApplicationMutationBinding<Schema>>::DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = <Reviewed as ApplicationMutationBinding<Schema>>::ScopeBinding;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "document-retention.correctable-reviewed.v1";
    const HANDLER_IDENTITY: &'static str = "document-retention.correctable-reviewed-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "document-retention.correctable-reviewed-command.v1";
    const REQUIRES_WORKFLOW_AUTHORITY: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        Reviewed::workflow_execution_requirements()
    }
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
impl ApplicationMutationAdmissionBinding<Schema> for CorrectableRetentionBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationIntent<Schema> for CorrectableRetentionIntent {
    type Binding = CorrectableRetentionBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.input
    }
    fn scope_binding(
        &self,
    ) -> <CorrectableRetentionBinding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        ReviewedSetRetentionIntent {
            input: self.input.clone(),
        }
        .scope_binding()
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema.operation_projection_work_budget(CorrectableRetention::reference(), 8)
}
