//! The guarded counterpart of the ordinary retention write used by workflow courts.

use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationMutationBinding, ApplicationMutationIntent,
            ApplicationWorkflowExecutionRequirements, NoApplicationMutationOutputs,
            NoApplicationMutationSource,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef, EqualityPredicate,
            NoApplicationUnit, ReadOnly, U64ApplicationValueBinding,
        },
    },
    primary_graph::WorthQueryInvariantMutationTarget,
};

use super::super::schema::{
    Document, DocumentFacts, DocumentIdentityField, DocumentPrincipalBinding,
    DocumentRetentionSchema, ExternalMapping, Principal, SetRetention, SetRetentionInputBinding,
};
use super::super::workflow::WorkflowApprovalCapability;
use super::{
    DocumentRetentionWritten, SetRetentionBinding, SetRetentionDenial, SetRetentionInput,
    SetRetentionScope,
};

#[derive(Clone, Debug)]
pub struct ExplicitReviewedSetRetentionIntent {
    pub input: SetRetentionInput,
}

pub struct ExplicitReviewedSetRetentionBinding;

impl ApplicationMutationBinding<DocumentRetentionSchema> for ExplicitReviewedSetRetentionBinding {
    type Input = SetRetentionInput;
    type InputBinding = SetRetentionInputBinding;
    type Result = DocumentRetentionWritten;
    type ResultBinding = super::DocumentRetentionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = SetRetention;
    type Decision = WorthQueryInvariantMutationTarget<DocumentRetentionSchema, Document>;
    type Denial = SetRetentionDenial;
    type DenialBinding = super::SetRetentionDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = SetRetentionScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str =
        "worth.query.certification.document-retention.explicit-reviewed-set-binding.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.document-retention.explicit-reviewed-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.document-retention.explicit-reviewed-command.v1";
    const REQUIRES_WORKFLOW_AUTHORITY: bool = true;
    const CANDIDATES:
        worth_query_decl::facade::application_operation::ApplicationCandidateContract =
        SetRetentionBinding::CANDIDATES;

    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::requiring_approval::<WorkflowApprovalCapability>()
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
        DocumentIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<DocumentRetentionSchema> for ExplicitReviewedSetRetentionIntent {
    type Binding = ExplicitReviewedSetRetentionBinding;

    fn input(&self) -> &SetRetentionInput {
        &self.input
    }

    fn scope_binding(&self) -> SetRetentionScope {
        SetRetentionScope::new(
            DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}

impl
    worth_query_decl::facade::application_operation::ApplicationMutationAdmissionBinding<
        DocumentRetentionSchema,
    > for ExplicitReviewedSetRetentionBinding
{
    type Admission =
        worth_query_decl::facade::application_operation::ApplicationConventionalAdmission;
}
