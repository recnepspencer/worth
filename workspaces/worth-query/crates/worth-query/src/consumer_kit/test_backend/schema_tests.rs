use super::{
    in_memory_test_runtime, WorthQueryTestBackendError, WorthQueryTestBackendErrorKind,
    WorthQueryTestBackendSchema,
};
use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};

#[test]
fn in_memory_test_runtime_validates_schema_denials_before_workspace_creation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    assert_schema_error(
        title_schema()
            .aspect("title.value", "title.value")
            .expect("first aspect should be accepted")
            .aspect("title.value", "task_title"),
        WorthQueryTestBackendErrorKind::DuplicateAspectLabel,
    );
    assert_schema_error(
        title_schema()
            .aspect_contract(string_contract("task_title", 2, &["value"]))
            .expect("second contract should admit")
            .aspect("title.value", "title.value")
            .expect("first aspect should be accepted")
            .aspect("task_title.value", "title.value"),
        WorthQueryTestBackendErrorKind::DuplicateProjectionPath,
    );
    assert_schema_error(
        title_schema_for_collection(" ").aspect("title.value", "title.value"),
        WorthQueryTestBackendErrorKind::BlankCollectionName,
    );
    assert_schema_error(
        title_schema().aspect(" ", "title.value"),
        WorthQueryTestBackendErrorKind::BlankAspectLabel,
    );
    assert_schema_error(
        title_schema().aspect("title.value", " "),
        WorthQueryTestBackendErrorKind::BlankProjectionPath,
    );
    assert_schema_error(
        title_schema().aspect("title.value", "title value"),
        WorthQueryTestBackendErrorKind::InvalidProjectionPath,
    );
    assert_schema_error(
        WorthQueryTestBackendSchema::single_collection("Task").aspect("title.value", "title.value"),
        WorthQueryTestBackendErrorKind::MissingAspectContract,
    );
    assert_schema_error(
        title_schema().aspect("title.undeclared", "title.undeclared"),
        WorthQueryTestBackendErrorKind::UndeclaredAspectField,
    );
    assert_eq!(
        workspace_build_error_kind(in_memory_test_runtime().workspace(
            request_execution,
            "consumer-kit.test-backend.missing-schema"
        )),
        WorthQueryTestBackendErrorKind::MissingSchema
    );
    assert_eq!(
        workspace_build_error_kind(
            in_memory_test_runtime()
                .with_schema(WorthQueryTestBackendSchema::single_collection("Task"))
                .workspace(request_execution, "consumer-kit.test-backend.empty-schema")
        ),
        WorthQueryTestBackendErrorKind::EmptyAspectSet
    );
}

#[test]
fn equivalent_contracts_converge_and_conflicting_contracts_deny() {
    let contract = string_contract("title", 1, &["value"]);
    let schema = WorthQueryTestBackendSchema::single_collection("Task")
        .aspect_contract(contract.clone())
        .unwrap()
        .aspect_contract(contract)
        .unwrap();
    assert_eq!(schema.contracts().count(), 1);

    assert_schema_error(
        schema.aspect_contract(string_contract("title", 2, &["value"])),
        WorthQueryTestBackendErrorKind::ConflictingAspectContract,
    );
}

fn title_schema() -> WorthQueryTestBackendSchema {
    title_schema_for_collection("Task")
}

fn title_schema_for_collection(collection: &str) -> WorthQueryTestBackendSchema {
    WorthQueryTestBackendSchema::single_collection(collection)
        .aspect_contract(string_contract("title", 1, &["value"]))
        .expect("title contract should admit")
}

fn string_contract(aspect: &str, identity: u64, fields: &[&str]) -> AspectContract {
    let fields = fields.iter().map(|field| {
        FieldDeclaration::new(
            FieldKey::new(*field).unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()
    });
    AspectContract::struct_aspect(
        AspectKey::new(aspect).unwrap(),
        AspectIdentity(identity),
        AspectContractRevision(1),
        StructAspectShape::new(fields).unwrap(),
    )
}

fn assert_schema_error(
    result: Result<WorthQueryTestBackendSchema, WorthQueryTestBackendError>,
    expected: WorthQueryTestBackendErrorKind,
) {
    match result {
        Ok(_) => panic!("schema declaration should fail"),
        Err(error) => assert_eq!(error.kind(), expected),
    }
}

fn workspace_build_error_kind(
    result: Result<crate::runtime::WorthQueryWorkspace, WorthQueryTestBackendError>,
) -> WorthQueryTestBackendErrorKind {
    match result {
        Ok(_) => panic!("workspace build should fail"),
        Err(error) => error.kind(),
    }
}
