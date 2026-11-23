//! Context and namespace spellings declare exactly what hand-written
//! clauses declare; a stated clause overrides the context.
use super::read::{
    self, ParametersBinding, ReadBinding, ReadTask, RequestBinding, RowBinding, TaskQuery,
};
use super::schema::*;
use super::tasks::{
    rename_definition, Rename, RenameBinding, RenameInputBinding, RenameTask, RENAME,
};
use std::any::TypeId;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*, worth_query_identity,
    worth_query_identity_namespace, worth_query_mutation_binding, worth_query_operation,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::*;

/// Every clause written out, with identities spelled independently.
/// `ExplicitRename<1>` differs only in its binding identity. Idempotency and
/// handler identities live on the descriptor, not the canonical schema, so the
/// parity court compares them there.
pub struct ExplicitRename<const KEY: u8 = 0>;
impl<const KEY: u8> ApplicationMutationAdmissionBinding<Schema> for ExplicitRename<KEY> {
    type Admission = ApplicationConventionalAdmission;
}
impl<const KEY: u8> ApplicationMutationBinding<Schema> for ExplicitRename<KEY> {
    type Input = Rename;
    type InputBinding = RenameInputBinding;
    type Result = ();
    type ResultBinding = BodyResultBinding;
    type IdempotencyKey = u64;
    type Operation = RenameTask;
    type Decision = WorthQueryOperationBodyDecision<Schema, Task>;
    type Denial = WorthQueryOperationBodyDenial;
    type DenialBinding = BodyDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<TaskKey>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = match KEY {
        0 => "single-source.tasks.rename.v1.binding",
        _ => "single-source.tasks.rename.v1.other-binding",
    };
    const HANDLER_IDENTITY: &'static str = "single-source.tasks.rename.v1.body";
    const IDEMPOTENCY_IDENTITY: &'static str = "single-source.board.v1.idempotency";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> TaskKey {
        TaskKey::at("TaskKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
}

pub struct ExplicitRead;
impl ApplicationQueryBinding<Schema> for ExplicitRead {
    type Input = ReadTask;
    type InputBinding = RequestBinding;
    type Query = TaskQuery;
    type ParameterBinding = ParametersBinding;
    type ResultBinding = RowBinding;
    type ScopeBinding = ApplicationQueryFieldScope<TaskKey>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "single-source.tasks.read.v1.binding";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(4);
    fn scope_field() -> TaskKey {
        TaskKey::at("TaskKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
}

fn board() -> ApplicationSchemaDeclarationBuilder<Schema> {
    ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Board)
        .expect("the board fragment lowers")
        .principal_binding(PrincipalBinding::reference())
}
fn with_mutation<B: ApplicationMutationBinding<Schema>>() -> ApplicationSchemaDeclaration<Schema> {
    board()
        .declared_operation(rename_definition())
        .application_mutation_binding::<B>()
        .build()
        .expect("the rename declares")
}
fn with_query<B: ApplicationQueryBinding<Schema>>() -> ApplicationSchemaDeclaration<Schema> {
    board()
        .application_query(read::definition())
        .application_query_binding::<B>()
        .build()
        .expect("the read declares")
}

#[test]
fn a_context_mutation_binding_declares_the_same_schema_as_explicit_clauses() {
    let (context, explicit) = (
        with_mutation::<RenameBinding>(),
        with_mutation::<ExplicitRename<0>>(),
    );
    assert_eq!(context.erased(), explicit.erased());
    let (context, explicit) = (
        RenameBinding::descriptor(),
        ExplicitRename::<0>::descriptor(),
    );
    assert_eq!(context.description(), explicit.description());
    assert_eq!(context.principal(), explicit.principal());
    assert_eq!(
        context.idempotency().identity(),
        explicit.idempotency().identity()
    );
    assert_eq!(
        context.idempotency().key_type(),
        explicit.idempotency().key_type()
    );
    assert_eq!(context.handler().identity(), explicit.handler().identity());
    assert_eq!(
        context.denial_binding_type(),
        explicit.denial_binding_type()
    );
}

/// An intent that carries the shared `Rename` input beside its own scope key.
pub struct RenameRequest {
    shared: Rename,
    task: u64,
}
fn request_input(request: &RenameRequest) -> &Rename {
    &request.shared
}
fn request_task(request: &RenameRequest) -> u64 {
    request.task
}
worth_query_mutation_binding!(pub SharedRenameBinding for Rename, schema Schema, context BoardContext,
    intent RenameRequest => request_input,
    identity worth_query_identity!(RENAME, binding), input RenameInputBinding,
    operation RenameTask, result BodyResultBinding,
    decision WorthQueryOperationBodyDecision<Schema, Task>,
    handler identity worth_query_identity!(RENAME, body), program required,
    outputs NoApplicationMutationOutputs,
    scope TaskKey, field TaskKey::at("TaskKey"), value request_task,
);

#[test]
fn a_shared_input_binding_declares_the_same_schema_as_explicit_clauses() {
    let shared = with_mutation::<SharedRenameBinding>();
    assert_eq!(
        shared.erased(),
        with_mutation::<ExplicitRename<0>>().erased()
    );
    assert_ne!(
        shared.erased(),
        with_mutation::<ExplicitRename<1>>().erased()
    );
    let (shared, explicit) = (
        SharedRenameBinding::descriptor(),
        ExplicitRename::<0>::descriptor(),
    );
    assert_eq!(shared.description(), explicit.description());
    assert_eq!(shared.principal(), explicit.principal());
    assert_eq!(
        shared.idempotency().identity(),
        explicit.idempotency().identity()
    );
    assert_eq!(
        shared.idempotency().key_type(),
        explicit.idempotency().key_type()
    );
    assert_eq!(shared.handler().identity(), explicit.handler().identity());
    assert_eq!(shared.denial_binding_type(), explicit.denial_binding_type());
    let request = RenameRequest {
        shared: Rename {
            task: 3,
            title: "Shared".into(),
        },
        task: 7,
    };
    assert!(std::ptr::eq(request.input(), &request.shared));
    assert_eq!(request.scope_binding().into_field_parts(&0).1, 7);
}

#[test]
fn a_context_query_binding_declares_the_same_schema_as_explicit_clauses() {
    assert_eq!(
        with_query::<ReadBinding>().erased(),
        with_query::<ExplicitRead>().erased()
    );
    assert_eq!(
        ReadBinding::descriptor().principal(),
        ExplicitRead::descriptor().principal()
    );
}

#[test]
fn the_schema_comparison_detects_one_differing_clause() {
    let (context, other) = (
        with_mutation::<RenameBinding>(),
        with_mutation::<ExplicitRename<1>>(),
    );
    assert_ne!(context.erased(), other.erased());
    assert_ne!(context.identity(), other.identity());
}

/// An existing wire identity survives adoption of the namespace.
pub const RETITLE: ApplicationIdentityNamespace = worth_query_identity_namespace!(
    "single-source.tasks.retitle.v1",
    Binding => "board.retitle.mutation-binding.v1",
);
#[derive(Clone, Debug, WorthValue)]
pub struct Retitle {
    pub task: u64,
}
pub struct LegacyDenial;
worth_query_structured_value_binding!(pub RetitleInputBinding for Retitle {
    identity: worth_query_identity!(RETITLE, input)
});
worth_query_structured_value_binding!(pub LegacyDenialBinding for LegacyDenial {
    identity: "board.retitle.denial.v1"
});
worth_query_operation!(pub RetitleTask for Schema, input RetitleInputBinding);
worth_query_mutation_binding!(pub RetitleBinding for Retitle, schema Schema, context BoardContext,
    identity worth_query_identity!(RETITLE, binding), input RetitleInputBinding,
    operation RetitleTask, result BodyResultBinding,
    idempotency String, identity "board.retitle.key.v1",
    decision (), denial LegacyDenialBinding,
    handler identity worth_query_identity!(RETITLE, handler),
    outputs NoApplicationMutationOutputs,
    scope TaskKey, principal_field TaskKey::at("TaskKey"),
);

#[test]
fn stated_clauses_and_identity_overrides_win_over_derived_defaults() {
    let descriptor = RetitleBinding::descriptor();
    assert_eq!(descriptor.identity(), "board.retitle.mutation-binding.v1");
    assert_eq!(
        descriptor.handler().identity(),
        "single-source.tasks.retitle.v1.handler"
    );
    assert_eq!(
        descriptor.input_identity().as_str(),
        "single-source.tasks.retitle.v1.input"
    );
    assert_eq!(descriptor.idempotency().identity(), "board.retitle.key.v1");
    assert_eq!(descriptor.idempotency().key_type(), TypeId::of::<String>());
    assert_eq!(
        descriptor.denial_binding_type(),
        TypeId::of::<LegacyDenialBinding>()
    );
    assert_eq!(
        descriptor.principal(),
        RenameBinding::descriptor().principal()
    );
}
