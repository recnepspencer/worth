//! Read-only assertions over the actual Home's owner-issued Native/World basis.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    ApplicationHomeBinding, AspectKey, AspectValue, ContractValidatedAspectValueView, FieldKey,
    HomeIdentity,
};
use worth_relational::facade::{
    history::CommitId,
    runtime::{EntityReadRecord, RelationalRuntime},
};
use worth_runtime_world::facade::ProductBranchObservation;
use worth_store_application::*;

fn field<'a>(row: &'a EntityReadRecord, aspect: &str, field: &str) -> &'a AspectValue {
    let value = row
        .authoritative_aspect_state
        .as_ref()
        .unwrap()
        .get(&AspectKey::new(aspect).unwrap())
        .unwrap();
    let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
        panic!("actual contract is struct");
    };
    fields.get(&FieldKey::new(field).unwrap()).unwrap()
}
fn number(row: &EntityReadRecord, field_name: &str) -> u64 {
    let AspectValue::UInt64(value) = field(row, "bootstrap-publication", field_name) else {
        panic!("required u64");
    };
    *value
}

pub(super) fn completed(
    native: &RelationalRuntime,
    root: &ProductBranchObservation,
    expected: &ApplicationHomeBinding,
    execution: &ExecutionRequest<'_, '_>,
) {
    let observation = root.basis().relational_basis().observation();
    let view = native
        .read_truth()
        .project_observation(&observation)
        .unwrap();
    let pin = native
        .read_truth()
        .pin_view_read_capability(&view, &[], execution)
        .unwrap();
    let registry = pin.schema();
    let authority = registry.authority_snapshot();
    let completion = authority
        .entity_kinds
        .iter()
        .find(|entry| entry.kind_name == "worth-query-bootstrap-publication")
        .unwrap();
    let registration = registry.entity_registration(completion.kind_id).unwrap();
    assert_eq!(registration.aspect_contract_declarations.aspects.len(), 1);
    assert_eq!(
        registration.aspect_contract_declarations.aspects[0]
            .contract
            .key()
            .as_str(),
        "bootstrap-publication"
    );
    let rows = view
        .bounded_entities_of_kind(execution, completion.kind_id, 128)
        .unwrap()
        .unwrap();
    let [row] = rows.records() else {
        panic!("exactly one genuine completion row");
    };
    for (name, expected) in [
        ("format-version", 1),
        ("principal-bindings", 2),
        ("identity-indexes", 1),
        ("application-equality-indexes", 5),
        ("policy-entities", 2),
        ("policy-relations", 0),
    ] {
        assert_eq!(number(row, name), expected, "literal aggregate {name}");
    }
    let terminal_id = CommitId(number(row, "terminal-bootstrap-commit"));
    let terminal = native
        .history()
        .immutable_commit_receipt(terminal_id)
        .unwrap();
    let selected = native
        .history()
        .branch_head_for_observation(&observation)
        .unwrap()
        .unwrap();
    assert_eq!(selected.parents, [terminal.commit_id]);
    assert_eq!(selected.branch_id, terminal.branch_id);
    assert_eq!(row.created_at_version, selected.version_id);
    assert_ne!(selected.commit_id, terminal.commit_id);
    assert_eq!(terminal.version_id.as_u64(), 3);
    assert_eq!(selected.version_id.as_u64(), 4);
    let old = native
        .read_truth()
        .bounded_visible_entities_of_kind(execution, completion.kind_id, terminal.version_id, 128)
        .unwrap()
        .unwrap();
    assert!(
        old.records().is_empty(),
        "completion is absent at the real terminal seed basis"
    );

    let mut binding = String::from("WQHB1\n");
    let mut item = |value: &str| {
        use std::fmt::Write;
        write!(binding, "{}:{value},", value.len()).unwrap();
    };
    item(&expected.package().render_hex());
    item(&expected.schema().render_hex());
    item(expected.selected().identity());
    item(&expected.selected().revision().render_hex());
    item(&expected.roster().len().to_string());
    for program in expected.roster() {
        item(program.identity());
        item(&program.revision().render_hex());
    }
    assert_eq!(
        field(row, "bootstrap-publication", "binding"),
        &AspectValue::String(binding.into())
    );

    let activation = authority
        .entity_kinds
        .iter()
        .find(|entry| entry.kind_name == "worth-query-program-activation")
        .unwrap();
    let activations = view
        .bounded_entities_of_kind(execution, activation.kind_id, 128)
        .unwrap()
        .unwrap();
    let [activation] = activations.records() else {
        panic!("one actual activation");
    };
    assert_eq!(activation.created_at_version.as_u64(), 1);
    assert_eq!(
        field(activation, "activation", "program-revision"),
        &AspectValue::String(expected.selected().revision().render_hex().into())
    );
}

pub(super) fn failed_close(close: ApplicationHomeCloseOutcome, identity: HomeIdentity) {
    let custody = match close {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason: ApplicationHomeCloseDenial::PhysicalInspectionRequired,
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
        } => custody,
        ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("actual refusal: {reason:?}")
        }
        ApplicationHomeCloseOutcome::Closed { .. } => {
            panic!("uncertain physical effect cannot claim Closed")
        }
    };
    assert_eq!(custody.home_identity(), identity);
    assert!(custody.inspection().is_some());
    let actual = custody
        .into_physical_close_outcome()
        .unwrap_or_else(|_| panic!("actual physical inspection remains owned"));
    assert!(actual.requires_inspection());
    drop(actual);
}
