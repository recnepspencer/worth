//! The company workflow control step the vocabulary requires: definition
//! authoring, a `WORKFLOW_CONTROL` binding the kernel authorizes through its
//! own company capability; no handler serves it. Steps on an instance are
//! `subject_control`'s.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{worth_query_operation, worth_query_structured_value_binding};

use super::grants::{self, AuthorCapability, Projection};
use super::schema::*;

/// The company a control step acts on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompanyControl {
    pub company: String,
}

worth_query_structured_value_binding!(pub CompanyControlInput for CompanyControl {
    identity: "workflow-birth-time.dx.control.input.v1"
});

type ContextBinding = <Desktop as ApplicationBindingContext<BirthSchema>>::PrincipalBinding;
type ContextMapping = <Desktop as ApplicationBindingContext<BirthSchema>>::Mapping;
type ContextPrincipal = <Desktop as ApplicationBindingContext<BirthSchema>>::Principal;
type ContextIdentity = <Desktop as ApplicationBindingContext<BirthSchema>>::PrincipalIdentity;
type ContextIdentityBinding =
    <Desktop as ApplicationBindingContext<BirthSchema>>::PrincipalIdentityBinding;

macro_rules! workflow_control {
    ($Operation:ident, $Binding:ident, $Capability:ty, $action:expr, $name:literal) => {
        worth_query_operation!(pub $Operation for BirthSchema, input CompanyControlInput);

        pub struct $Binding;

        impl ApplicationMutationBinding<BirthSchema> for $Binding {
            type Input = CompanyControl;
            type InputBinding = CompanyControlInput;
            type Result = ();
            type ResultBinding = UnitApplicationValueBinding;
            type IdempotencyKey = u64;
            type Operation = $Operation;
            type Decision = ();
            type Denial =
                <BodyDenial as ApplicationStructuredValueBinding>::Value;
            type DenialBinding = BodyDenial;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding = ApplicationMutationFieldScope<CompanyId>;
            type PrincipalBinding = ContextBinding;
            type Mapping = ContextMapping;
            type Principal = ContextPrincipal;
            type PrincipalIdentity = ContextIdentity;
            type PrincipalIdentityBinding = ContextIdentityBinding;
            type SourceExpectation = NoApplicationMutationSource;

            const IDENTITY: &'static str = concat!("workflow-birth-time.dx.", $name, ".binding.v1");
            const HANDLER_IDENTITY: &'static str =
                concat!("workflow-birth-time.dx.", $name, ".handler.v1");
            const IDEMPOTENCY_IDENTITY: &'static str =
                concat!("workflow-birth-time.dx.", $name, ".key.v1");
            const REQUIRES_APPLICATION_PROGRAM: bool = true;
            const WORKFLOW_CONTROL: bool = true;

            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
            }

            fn scope_field() -> CompanyId {
                CompanyFacts::company_id()
            }

            fn principal_binding() -> ApplicationPrincipalBindingRef<
                BirthSchema,
                ContextBinding,
                ContextMapping,
                ContextPrincipal,
                ContextIdentity,
                ContextIdentityBinding,
            > {
                <Desktop as ApplicationBindingContext<BirthSchema>>::principal_binding()
            }
        }

        impl ApplicationMutationAdmissionBinding<BirthSchema> for $Binding {
            type Admission = ApplicationCapabilityAdmission<$Capability>;
        }

        impl ApplicationCapabilityMutationBinding<BirthSchema> for $Binding {
            type Capability = $Capability;
        }

        impl ApplicationCapabilityRequest<BirthSchema, $Capability> for CompanyControl {
            type Scope = Company;
            type Context = grants::CompanyWorkflowContext;
            fn capability_request(
                &self,
            ) -> Result<Projection, ApplicationCapabilityRequestProjectionDenial> {
                grants::request(&self.company, $action)
            }
        }
    };
}

workflow_control!(
    AuthorOp,
    AuthorBinding,
    AuthorCapability,
    grants::AUTHOR,
    "author"
);

/// Authors a definition over one company.
#[derive(Clone, Debug)]
pub struct Author(pub CompanyControl);

impl ApplicationMutationIntent<BirthSchema> for Author {
    type Binding = AuthorBinding;
    fn input(&self) -> &CompanyControl {
        &self.0
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<CompanyId> {
        ApplicationMutationFieldScope::new(CompanyFacts::company_id(), self.0.company.clone())
    }
}

fn pure<Op>(
    operation: ApplicationOperationRef<BirthSchema, Op, CompanyControl>,
) -> ApplicationOperationDefinition<BirthSchema, Op, CompanyControl>
where
    Op: ApplicationOperationMarkerIdentity<BirthSchema, InputBinding = CompanyControlInput>,
{
    operation
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
}

/// Declares the control operations, their company read and their bindings.
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BirthSchema>,
) -> ApplicationSchemaDeclarationBuilder<BirthSchema> {
    let author = AuthorOp::reference();
    schema
        .operation(pure(author))
        .operation_projection_work_budget(author, 64)
        .operation_read_field(author, CompanyFacts::company_id())
        .application_mutation_binding::<AuthorBinding>()
}
