//! The steps on an instance: start (which also authorizes migration), the
//! `review` proposal slot and advance. A step acts on its instance's
//! subject, so each one is scoped to
//! the request and authorized by the same rooted subject contract as the
//! subject read, under its own grant action.
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{worth_query_capability, worth_query_operation};

use super::disclosure::{subject_contract, RequestSubject, RequestSubjectBinding, SubjectAction};
use super::grants::{ADVANCE, REVIEW, START};
use super::schema::*;

type ContextBinding = <Desktop as ApplicationBindingContext<BirthSchema>>::PrincipalBinding;
type ContextMapping = <Desktop as ApplicationBindingContext<BirthSchema>>::Mapping;
type ContextPrincipal = <Desktop as ApplicationBindingContext<BirthSchema>>::Principal;
type ContextIdentity = <Desktop as ApplicationBindingContext<BirthSchema>>::PrincipalIdentity;
type ContextIdentityBinding =
    <Desktop as ApplicationBindingContext<BirthSchema>>::PrincipalIdentityBinding;
type Scope = ApplicationMutationFieldScope<RequestId>;

macro_rules! subject_control {
    ($Operation:ident, $Binding:ident, $Capability:ident, $Intent:ident, $action:expr, $name:literal) => {
        worth_query_capability!(pub $Capability in BirthSchema);
        worth_query_operation!(pub $Operation for BirthSchema, input RequestSubjectBinding);

        impl SubjectAction for $Capability {
            const ACTION: &'static str = $action;
        }

        pub struct $Binding;

        impl ApplicationMutationBinding<BirthSchema> for $Binding {
            type Input = RequestSubject;
            type InputBinding = RequestSubjectBinding;
            type Result = ();
            type ResultBinding = UnitApplicationValueBinding;
            type IdempotencyKey = u64;
            type Operation = $Operation;
            type Decision = ();
            type Denial = <BodyDenial as ApplicationStructuredValueBinding>::Value;
            type DenialBinding = BodyDenial;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding = Scope;
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

            fn scope_field() -> RequestId {
                RequestFacts::request_id()
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

        /// One step on the instance whose subject is this request.
        #[derive(Clone, Debug)]
        pub struct $Intent(pub RequestSubject);

        impl ApplicationMutationIntent<BirthSchema> for $Intent {
            type Binding = $Binding;
            fn input(&self) -> &RequestSubject {
                &self.0
            }
            fn scope_binding(&self) -> Scope {
                Scope::new(RequestFacts::request_id(), self.0.request)
            }
        }
    };
}

subject_control!(
    StartOp,
    StartBinding,
    StartCapability,
    StartRequest,
    START,
    "start"
);
subject_control!(
    ReviewOp,
    ReviewBinding,
    ReviewCapability,
    Review,
    REVIEW,
    "review"
);
subject_control!(
    AdvanceOp,
    AdvanceBinding,
    AdvanceCapability,
    AdvanceRequest,
    ADVANCE,
    "advance"
);

/// One step's operation over the request, its reads and its binding.
fn step<Op, Binding>(
    schema: ApplicationSchemaDeclarationBuilder<BirthSchema>,
    operation: ApplicationOperationRef<BirthSchema, Op, RequestSubject>,
) -> ApplicationSchemaDeclarationBuilder<BirthSchema>
where
    Op: ApplicationOperationMarkerIdentity<BirthSchema, InputBinding = RequestSubjectBinding>
        + 'static,
    Binding: ApplicationMutationBinding<BirthSchema>,
{
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 128)
        .operation_read_field(operation, CompanyFacts::company_id())
        .operation_read_field(operation, RequestFacts::request_id())
        .operation_read_field(operation, RequestFacts::company_code())
        .application_mutation_binding::<Binding>()
}

/// Declares the three steps, their request reads, bindings and contracts.
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BirthSchema>,
) -> ApplicationSchemaDeclarationBuilder<BirthSchema> {
    let (start, review, advance) = (
        StartOp::reference(),
        ReviewOp::reference(),
        AdvanceOp::reference(),
    );
    let schema = step::<_, StartBinding>(schema, start);
    let schema = step::<_, ReviewBinding>(schema, review);
    step::<_, AdvanceBinding>(schema, advance)
        .capability(subject_contract(StartCapability::reference(), start))
        .capability(subject_contract(ReviewCapability::reference(), review))
        .capability(subject_contract(AdvanceCapability::reference(), advance))
}
