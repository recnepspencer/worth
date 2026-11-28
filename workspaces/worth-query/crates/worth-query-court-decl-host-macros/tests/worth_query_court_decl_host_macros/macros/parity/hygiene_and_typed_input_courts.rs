//! Focused Round 3 courts for:
//! - `#[serde(rename_all = "lowercase")]` identity semantics on mixed-case struct fields
//! - `pub` action structs with private `Serialize` member types (zero `E0446`)
//! - authoritative `InputBinding` verification in `ApplicationInputField::for_binding`
//!   and `TypedApplicationOperationBodyBuilder` (rejecting fieldless/mismatched/cross-binding
//!   selectors with `InvalidInputMember`)
use super::super::actions::{
    Rename, RenameInput, RenameTask, Reschedule, RescheduleInput,
};
use super::super::schema::*;
use worth_foundational::facade::ScalarAspectType;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_operation::ApplicationEncodedInput;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity_namespace, worth_query_operation, worth_query_structured_value_binding,
    WorthQueryAction,
};

const LOWER_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.lowercase-wire.v1");
const PRIVATE_MEMBER_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.private-member.v1");
const NO_FIELDS_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.parity.no-fields.v1");

// --- Fix 1: `#[serde(rename_all = "lowercase")]` is identity for struct fields ---
#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[serde(rename_all = "lowercase")]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = LOWER_NS,
    input = LowercaseWireInput,
    operation = LowercaseWireOp,
    binding = LowercaseWireBinding,
    body,
    reads(Task),
    writes(Task),
)]
#[allow(non_snake_case)]
pub struct LowercaseWireAction {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub fooBar: String,
    pub foobar: String,
}

#[test]
fn serde_rename_all_lowercase_preserves_mixed_case_struct_fields_matching_serde() {
    let sample = LowercaseWireAction {
        task: 1,
        fooBar: "camel".into(),
        foobar: "lower".into(),
    };
    // Verify WORTH's real canonical input selector reads both distinct Serde keys
    // (`fooBar` -> "camel" and `foobar` -> "lower") using the derived typed accessors!
    let encoded = ApplicationEncodedInput::<LowercaseWireInput>::encode(sample)
        .expect("lowercase-renamed action encodes canonically");
    let canonical = encoded
        .canonical_input()
        .expect("body action retains input");
    assert_eq!(LowercaseWireAction::fooBar_input().member(), "fooBar");
    assert_eq!(LowercaseWireAction::foobar_input().member(), "foobar");
    assert_eq!(
        canonical
            .scalar_member(
                LowercaseWireAction::fooBar_input().member(),
                ScalarAspectType::String,
            )
            .expect("fooBar selects from canonical wire")
            .into_value(),
        StringApplicationValueBinding::encode(&"camel".to_owned()).unwrap()
    );
    assert_eq!(
        canonical
            .scalar_member(
                LowercaseWireAction::foobar_input().member(),
                ScalarAspectType::String,
            )
            .expect("foobar selects from canonical wire")
            .into_value(),
        StringApplicationValueBinding::encode(&"lower".to_owned()).unwrap()
    );

    let _ = LowercaseWireOp::reference()
        .typed_body()
        .assign_input(TaskFacts::title(), LowercaseWireAction::fooBar_input())
        .finish_pure_definition()
        .expect("lowercase-renamed typed selector admits");
}

// --- Fix 2a: `pub` action struct with private `Serialize` member compiles with zero `E0446` ---
#[derive(Clone, Debug, serde::Serialize)]
struct PrivateDetail {
    flag: bool,
}

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = PRIVATE_MEMBER_NS,
    input = PublicWithPrivateInput,
    operation = PublicWithPrivateOp,
    binding = PublicWithPrivateBinding,
    body,
    reads(Task),
    writes(Task),
)]
pub struct PublicActionWithPrivateMember {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub title: String,
    detail: PrivateDetail,
}

// --- Fix 2b: Fieldless / mismatched / cross-binding selectors are refused fail-closed ---
fn fixed_inbox(_: &NoFieldsAction) -> u64 {
    1
}

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(
    schema = Schema,
    context = BoardContext,
    namespace = NO_FIELDS_NS,
    input = NoFieldsInput,
    operation = NoFieldsOp,
    binding = NoFieldsBinding,
    scope = TaskKey,
    scope_value = fixed_inbox,
    body,
    reads(Task),
    writes(Task),
)]
pub struct NoFieldsAction;

// A separate binding for `Rename` that declares NO closed field inventory,
// plus a second binding that declares the EXACT SAME field shape (`title: String`),
// proving `TypedApplicationOperationBodyBuilder` checks issuing binding identity
// as well as canonical member shape.
worth_query_structured_value_binding!(pub UnshapedRenameInput for Rename {
    identity: "macros.parity.unshaped-rename.input.v1",
    capture_input: true,
});
worth_query_operation!(
    pub UnshapedRenameOp for Schema, input UnshapedRenameInput;
    access { read_fields: [TaskKey::at("TaskKey")]; };
    reads [Task];
    writes [Task]
);
worth_query_structured_value_binding!(pub SameShapeRenameInput for Rename {
    identity: "macros.parity.same-shape-rename.input.v1",
    capture_input: true,
    fields: [required("task", task: u64), required("title", title: String)],
});
worth_query_operation!(
    pub SameShapeRenameOp for Schema, input SameShapeRenameInput;
    access { read_fields: [TaskKey::at("TaskKey")]; };
    reads [Task];
    writes [Task]
);

#[test]
fn private_member_compiles_without_e0446_and_unverified_selectors_are_refused() {
    let sample = PublicActionWithPrivateMember {
        task: 7,
        title: "visible".into(),
        detail: PrivateDetail { flag: true },
    };
    assert!(sample.detail.flag);
    assert_eq!(
        PublicActionWithPrivateMember::title_input().member(),
        "title"
    );
    assert_eq!(
        PublicActionWithPrivateMember::detail_input().member(),
        "detail"
    );
    let _ = PublicWithPrivateOp::reference()
        .typed_body()
        .assign_input(
            TaskFacts::title(),
            PublicActionWithPrivateMember::title_input(),
        )
        .finish()
        .expect("public field on action with private member admits");

    // Fieldless struct refuses fabricated selector.
    assert_eq!(
        ApplicationInputField::<NoFieldsAction, String>::for_binding::<NoFieldsInput>("invented"),
        Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );
    assert_eq!(
        ApplicationOptionalInputField::<NoFieldsAction, String>::for_binding::<NoFieldsInput>(
            "invented"
        ),
        Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );

    // Mismatched member name, mismatched scalar type, and mismatched optionality all refuse.
    assert_eq!(
        ApplicationInputField::<Rename, String>::for_binding::<RenameInput>("missing_field"),
        Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );
    assert_eq!(
        ApplicationInputField::<Rename, u64>::for_binding::<RenameInput>("title"),
        Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );
    assert_eq!(
        ApplicationOptionalInputField::<Rename, String>::for_binding::<RenameInput>("title"),
        Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );
    assert_eq!(
        ApplicationInputField::<Reschedule, CanonicalDate>::for_binding::<RescheduleInput>("due"),
        Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );

    // Even a valid selector minted from `RenameInput` is rejected by `TypedApplicationOperationBodyBuilder`
    // when applied to an operation whose authoritative `InputBinding` (`UnshapedRenameInput`)
    // does not declare that canonical member.
    let valid_for_rename = Rename::title_input();
    assert!(RenameTask::reference()
        .typed_body()
        .assign_input(TaskFacts::title(), valid_for_rename)
        .finish()
        .is_ok());
    assert_eq!(
        UnshapedRenameOp::reference()
            .typed_body()
            .assign_input(TaskFacts::title(), valid_for_rename)
            .finish()
            .err(),
        Some(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );
    // Even a distinct binding with the IDENTICAL field shape (`SameShapeRenameInput`)
    // refuses a selector minted by `RenameInput`, and accepts only its own selector.
    assert_eq!(
        SameShapeRenameOp::reference()
            .typed_body()
            .assign_input(TaskFacts::title(), valid_for_rename)
            .finish()
            .err(),
        Some(ApplicationOperationBodyAuthoringDenial::InvalidInputMember)
    );
    let own_same_shape =
        ApplicationInputField::<Rename, String>::for_binding::<SameShapeRenameInput>("title")
            .expect("same-shape binding mints its own selector");
    assert_ne!(own_same_shape, valid_for_rename);
    assert!(SameShapeRenameOp::reference()
        .typed_body()
        .assign_input(TaskFacts::title(), own_same_shape)
        .finish()
        .is_ok());
}

// --- Fix 3: Item-level correction hygiene traps ---
