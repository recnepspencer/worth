//! A derived action whose namespace keeps an existing binding identity
//! declares exactly what the declaration macros and a hand-written binding
//! impl declare with every identity spelled out.
use super::super::schema::*;
use super::board;
use std::any::TypeId;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*,
    worth_query_identity_namespace, WorthQueryAction,
};
use worth_query_host::facade::primary_graph::*;

pub const RETITLE: ApplicationIdentityNamespace = worth_query_identity_namespace!(
    "macros.tasks.retitle.v1",
    Binding => "board.retitle.binding.legacy.v1",
);

#[derive(Clone, Debug, WorthValue, WorthQueryAction)]
#[worth_query(schema = Schema, context = BoardContext, namespace = RETITLE,
    input = RetitleInput, operation = RetitleTask, binding = RetitleBinding, delivery = Retitled,
    result = BodyResultBinding, decision = WorthQueryOperationBodyDecision<Schema, Task>,
    body, program, access(read_fields: [TaskKey::at("TaskKey")];), reads(Task))]
pub struct Retitle {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub title: String,
}

/// The same action, written with the declaration macros and trait impls.
/// `RetitleBinding<1>` differs only in its binding identity.
mod written {
    use super::*;
    use worth_query_decl::facade::{worth_query_operation, worth_query_structured_value_binding};

    worth_query_structured_value_binding!(pub RetitleInput for Retitle {
        identity: "macros.tasks.retitle.v1.input",
        capture_input: true
    });
    worth_query_operation!(pub RetitleTask for Schema, input RetitleInput;
        access { read_fields: [TaskKey::at("TaskKey")]; };
        reads [Task]
    );
    pub struct RetitleBinding<const KEY: u8 = 0>;
    impl<const KEY: u8> ApplicationMutationAdmissionBinding<Schema> for RetitleBinding<KEY> {
        type Admission = ApplicationConventionalAdmission;
    }
    impl<const KEY: u8> ApplicationMutationBinding<Schema> for RetitleBinding<KEY> {
        type Input = Retitle;
        type InputBinding = RetitleInput;
        type Result = ();
        type ResultBinding = BodyResultBinding;
        type IdempotencyKey = u64;
        type Operation = RetitleTask;
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
            0 => "board.retitle.binding.legacy.v1",
            _ => "board.retitle.binding.other.v1",
        };
        const HANDLER_IDENTITY: &'static str = "macros.tasks.retitle.v1.body";
        const IDEMPOTENCY_IDENTITY: &'static str = "macros.board.v1.idempotency";
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
    pub struct Retitled;
    impl ApplicationActionDelivery<Schema, RetitleBinding> for Retitled {
        type Mode = ApplicationSourceCompletion;
        const IDENTITY: &'static str = "macros.tasks.retitle.v1.completion";
    }

    pub fn declared<const KEY: u8>() -> ApplicationSchemaDeclaration<Schema> {
        let operation = RetitleTask::reference()
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish();
        board()
            .declared_operation(operation)
            .application_mutation_binding::<RetitleBinding<KEY>>()
            .build()
            .expect("the written action declares")
    }
}

fn derived() -> ApplicationSchemaDeclaration<Schema> {
    let operation = RetitleTask::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish();
    board()
        .declared_operation(operation)
        .application_mutation_binding::<RetitleBinding>()
        .build()
        .expect("the derived action declares")
}

#[test]
fn a_derived_action_keeps_its_overridden_wire_identity_and_declares_the_written_schema() {
    let (derived, written) = (derived(), written::declared::<0>());
    assert_eq!(derived.identity(), written.identity());
    assert_eq!(derived.erased(), written.erased());
    assert_ne!(derived.erased(), written::declared::<1>().erased());

    let (derived, written) = (
        RetitleBinding::descriptor(),
        written::RetitleBinding::<0>::descriptor(),
    );
    assert_eq!(derived.identity(), "board.retitle.binding.legacy.v1");
    assert_eq!(derived.description(), written.description());
    assert_eq!(derived.input_identity(), written.input_identity());
    assert_eq!(derived.principal(), written.principal());
    assert_eq!(derived.handler().identity(), written.handler().identity());
    assert_eq!(
        derived.idempotency().identity(),
        written.idempotency().identity()
    );
    assert_eq!(derived.idempotency().key_type(), TypeId::of::<u64>());
    assert_eq!(derived.denial_binding_type(), written.denial_binding_type());
    assert_eq!(
        <Retitled as ApplicationActionDelivery<Schema, RetitleBinding>>::IDENTITY,
        <written::Retitled as ApplicationActionDelivery<Schema, written::RetitleBinding>>::IDENTITY,
    );
}
