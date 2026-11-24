//! Parity for zero-string field selectors, typed authored-body builders,
//! `scope_value` actions, and smart `body` action defaults.
use super::super::schema::*;
use super::board;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, worth_query_identity,
    worth_query_identity_namespace, worth_query_mutation_binding, worth_query_operation,
    worth_query_structured_value_binding, WorthQueryAction,
};
use worth_query_host::facade::primary_graph::*;

const FIXED_ACTION: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.fixed-scope-action.v1");
const DEFAULT_BODY: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.default-body-action.v1");

fn inbox_task_key(_: &PinInboxTitle) -> u64 {
    1
}

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = FIXED_ACTION,
    input = PinInboxInput,
    operation = PinInboxTask,
    binding = PinInboxBinding,
    delivery = PinnedInbox,
    scope = TaskKey,
    scope_value = inbox_task_key,
    body,
    program,
    reads(Task),
)]
pub struct PinInboxTitle {
    pub expected: String,
    pub title: String,
    pub due: Option<CanonicalDate>,
}

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = DEFAULT_BODY,
    input = EditDueInput,
    operation = EditDueOp,
    binding = EditDueBinding,
    delivery = EditedDue,
    body,
    program,
    reads(Task),
)]
pub struct EditDue {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub due: Option<CanonicalDate>,
}

mod written {
    use super::*;

    #[derive(Clone, Debug, serde::Serialize)]
    pub struct WrittenPinInboxTitle {
        pub expected: String,
        pub title: String,
        pub due: Option<CanonicalDate>,
    }

    fn written_inbox_task_key(_: &WrittenPinInboxTitle) -> u64 {
        1
    }

    worth_query_structured_value_binding!(pub PinInboxInput for WrittenPinInboxTitle {
        identity: worth_query_identity!(FIXED_ACTION, input),
        capture_input: true
    });
    worth_query_operation!(pub PinInboxTask for Schema, input PinInboxInput;
        access { read_fields: [TaskKey::at("TaskKey")]; };
        reads [Task]
    );
    worth_query_mutation_binding!(pub PinInboxBinding for WrittenPinInboxTitle, schema Schema,
        context BoardContext,
        identity worth_query_identity!(FIXED_ACTION, binding),
        input PinInboxInput,
        operation PinInboxTask,
        result UnitApplicationValueBinding,
        decision WorthQueryOperationBodyDecision<Schema, Task>,
        handler identity worth_query_identity!(FIXED_ACTION, body),
        program required,
        outputs NoApplicationMutationOutputs,
        scope TaskKey, field TaskKey::at("TaskKey"), value written_inbox_task_key,
    );
    pub struct PinnedInbox;
    impl ApplicationActionDelivery<Schema, PinInboxBinding> for PinnedInbox {
        type Mode = ApplicationSourceCompletion;
        const IDENTITY: &'static str = worth_query_identity!(FIXED_ACTION, completion);
    }

    pub fn manual_pin_definition(
    ) -> ApplicationOperationDefinition<Schema, PinInboxTask, WrittenPinInboxTitle> {
        let body = PinInboxTask::reference()
            .body()
            .expect_input(Title::at("Title"), "expected")
            .unwrap()
            .assign_input(Title::at("Title"), "title")
            .unwrap()
            .assign_optional_input(Due::at("Due"), "due")
            .unwrap()
            .finish()
            .unwrap();
        PinInboxTask::reference()
            .definition()
            .no_external_effect()
            .no_aftermath()
            .finish()
            .with_body(body)
            .unwrap()
    }
}

#[test]
fn zero_string_selectors_match_manual_at_selections() {
    assert_eq!(TaskFacts::title().path(), Title::at("Title").path());
    assert_eq!(
        TaskFacts::TITLE.binding_recipe(),
        Title::at("Title").binding_recipe()
    );
    assert_eq!(
        TaskFactsValue::select_title().binding_recipe(),
        Title::at("Title").binding_recipe()
    );
    assert_eq!(
        TaskFactsValue::TITLE.binding_recipe(),
        Title::at("Title").binding_recipe()
    );
    assert_eq!(
        TaskFacts::due().binding_recipe(),
        Due::at("Due").binding_recipe()
    );
    assert_eq!(
        TaskFactsValue::select_key().binding_recipe(),
        TaskKey::at("TaskKey").binding_recipe()
    );
}

#[test]
fn typed_body_builder_and_scope_value_action_match_manual_declarations() {
    let typed_def = PinInboxTask::reference()
        .typed_body()
        .expect_input(TaskFacts::title(), PinInboxTitle::expected_input())
        .assign_input(TaskFacts::TITLE, PinInboxTitle::title_input())
        .assign_optional_input(TaskFactsValue::select_due(), PinInboxTitle::due_input())
        .finish_pure_definition()
        .expect("typed body admits");

    let derived = board()
        .declared_operation(typed_def)
        .application_mutation_binding::<PinInboxBinding>()
        .build()
        .expect("derived scope_value action declares");
    let written = board()
        .declared_operation(written::manual_pin_definition())
        .application_mutation_binding::<written::PinInboxBinding>()
        .build()
        .expect("written scope_value action declares");

    assert_eq!(derived.identity(), written.identity());
    assert_eq!(derived.erased(), written.erased());
    assert_eq!(
        PinInboxBinding::descriptor().description(),
        written::PinInboxBinding::descriptor().description()
    );
    assert_eq!(
        <PinnedInbox as ApplicationActionDelivery<Schema, PinInboxBinding>>::IDENTITY,
        <written::PinnedInbox as ApplicationActionDelivery<Schema, written::PinInboxBinding>>::IDENTITY
    );

    // Negative twin: omitting the optional due assignment changes canonical schema.
    let shorter_def = PinInboxTask::reference()
        .typed_body()
        .expect_input(TaskFacts::title(), PinInboxTitle::expected_input())
        .assign_input(TaskFacts::title(), PinInboxTitle::title_input())
        .finish_pure_definition()
        .expect("shorter body admits");
    let twin = board()
        .declared_operation(shorter_def)
        .application_mutation_binding::<PinInboxBinding>()
        .build()
        .expect("twin declares");
    assert_ne!(derived.erased(), twin.erased());

    // Fail-closed: assigning a read-only column first preserves the first denial
    // even when a later step is valid.
    let refusal = EditDueOp::reference()
        .typed_body()
        .assign_input(TaskFacts::key(), EditDue::task_input())
        .assign_optional_input(TaskFacts::due(), EditDue::due_input())
        .finish();
    assert_eq!(
        refusal.err(),
        Some(ApplicationOperationBodyAuthoringDenial::IncompatibleSourceBinding)
    );
}
