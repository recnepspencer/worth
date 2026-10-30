//! An explicit write intent deliberately permits reaffirming the current retention.
pub use super::super::retention_entry::{
    DocumentRetentionWritten, SetRetentionBinding, SetRetentionIntent,
};
pub use super::*;
pub use worth_query_host::facade::{declaration::application_operation::*, primary_graph::*, *};

worth_query_operation!(pub ReaffirmRetention for DocumentRetentionSchema, input SetRetentionInputBinding; access {
    read_fields: [DocumentIdentityField, DocumentRetentionField];
    writes: [DocumentRetentionField];
});
pub struct Binding;
pub type Base = SetRetentionBinding;
impl ApplicationMutationBinding<DocumentRetentionSchema> for Binding {
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = <Base as ApplicationMutationBinding<DocumentRetentionSchema>>::Result;
    type ResultBinding =
        <Base as ApplicationMutationBinding<DocumentRetentionSchema>>::ResultBinding;
    type IdempotencyKey = u64;
    type Operation = ReaffirmRetention;
    type Decision = <Base as ApplicationMutationBinding<DocumentRetentionSchema>>::Decision;
    type Denial = <Base as ApplicationMutationBinding<DocumentRetentionSchema>>::Denial;
    type DenialBinding =
        <Base as ApplicationMutationBinding<DocumentRetentionSchema>>::DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = <Base as ApplicationMutationBinding<DocumentRetentionSchema>>::ScopeBinding;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "record-history.reaffirm-retention.v1";
    const HANDLER_IDENTITY: &'static str = "record-history.reaffirm-retention.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "record-history.reaffirm-retention.command.v1";
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        DocumentRetentionSchema,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Base::scope_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Base::principal_binding()
    }
}
impl ApplicationMutationAdmissionBinding<DocumentRetentionSchema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
#[derive(Clone)]
pub struct Intent(pub SetRetentionInput);
impl ApplicationMutationIntent<DocumentRetentionSchema> for Intent {
    type Binding = Binding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <Binding as ApplicationMutationBinding<DocumentRetentionSchema>>::ScopeBinding {
        SetRetentionIntent {
            input: self.0.clone(),
        }
        .scope_binding()
    }
}
pub struct Handler;
impl OperationHandler<DocumentRetentionSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &SetRetentionInput,
        reader: &mut DecisionReader<'_, '_, '_, DocumentRetentionSchema, Binding>,
    ) -> HandlerResult<
        <Binding as ApplicationMutationBinding<DocumentRetentionSchema>>::Decision,
        <Binding as ApplicationMutationBinding<DocumentRetentionSchema>>::Denial,
    > {
        let result = reader
            .resolve_entity(DocumentIdentityField::reference(), input.identity.clone())
            .and_then(|document| {
                // The actual observed field is the write's native effect basis.
                reader.field(&document, DocumentRetentionField::reference())?;
                reader.mutation_target(&document)
            });
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn build_candidate(
        &self,
        input: &SetRetentionInput,
        target: <Binding as ApplicationMutationBinding<DocumentRetentionSchema>>::Decision,
        writer: &mut CandidateWriter<'_, DocumentRetentionSchema, Binding>,
    ) -> HandlerResult<
        DocumentRetentionWritten,
        <Binding as ApplicationMutationBinding<DocumentRetentionSchema>>::Denial,
    > {
        let result = writer.projected_entity(&target).and_then(|document| {
            writer.write_field(
                &document,
                DocumentRetentionField::reference(),
                input.retention_days,
            )
        });
        match result {
            Ok(()) => HandlerResult::Completed(DocumentRetentionWritten {
                retention_days: input.retention_days,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema
        .operation(
            ReaffirmRetention::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(
            ReaffirmRetention::reference(),
            DocumentIdentityField::reference(),
        )
        .operation_write(
            ReaffirmRetention::reference(),
            DocumentRetentionField::reference(),
        )
        .operation_read_field(
            ReaffirmRetention::reference(),
            DocumentRetentionField::reference(),
        )
        .application_mutation_binding::<Binding>()
}
