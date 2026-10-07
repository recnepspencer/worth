use super::*;
use crate::runtime::WorthQueryAuthoredAspectMutation;
use worth_foundational::facade::{
    aspects, AbsenceLaw, AspectContract, AspectContractRevision, AspectEquivalenceBasis,
    AspectEvolutionPolicy, AspectIdentity, AspectKey, AspectMaskContract, AspectValue,
    CanonicalFieldPath, ContractValidationInput, FieldDeclaration, FieldKey, FieldRequirement,
    ScalarAspectType, StructAspectShape, StructAspectValue,
};
use worth_relational::facade::runtime::InvariantCatalog;

mod native_contracts;

#[test]
fn memory_workspace_insert_aspects_tracks_changed_paths() {
    let mut workspace = WorthQueryMemoryWorkspace::collection(
        "Task",
        [
            aspect("identity.id", "identity.id"),
            aspect("title.value", "title.value"),
        ],
    )
    .expect("memory workspace should build");

    let receipt = workspace
        .insert_aspects(vec![
            WorthQueryAuthoredAspectMutation::new(touch("identity.id"), text("task-1"))
                .expect("identity aspect"),
            WorthQueryAuthoredAspectMutation::new(touch("title.value"), text("First task"))
                .expect("title aspect"),
        ])
        .expect("insert should succeed");

    assert_eq!(receipt.deltas.len(), 1);
    assert_eq!(receipt.deltas[0].kind, WorthQueryMutationKind::Created);
    assert_eq!(
        receipt.deltas[0].admitted_touched_aspects(),
        &[touch("identity.id"), touch("title.value")]
    );
    assert_eq!(workspace.entities().unwrap().len(), 1);
}

#[test]
fn runtime_receipt_identities_reject_equal_raw_projections_as_current_authority() {
    let mut workspace =
        WorthQueryMemoryWorkspace::collection("Task", [aspect("identity.id", "identity.id")])
            .expect("memory workspace should build");
    let receipt = workspace
        .insert_aspects(vec![WorthQueryAuthoredAspectMutation::new(
            touch("identity.id"),
            text("task-1"),
        )
        .expect("identity aspect")])
        .expect("insert should succeed");

    let copied_commit = WorthQueryCommitIdentity::from_relational_commit_id(
        receipt
            .commit_identity
            .relational_commit_id()
            .expect("runtime receipt commit retains relational projection"),
    );
    let copied_snapshot = WorthQuerySnapshotIdentity::from_relational_snapshot(
        receipt
            .snapshot_identity
            .relational_parts()
            .expect("runtime receipt snapshot retains relational projection"),
    );

    assert!(receipt
        .commit_identity
        .is_same_current_identity_as(&receipt.commit_identity.clone()));
    assert!(receipt
        .snapshot_identity
        .is_same_current_identity_as(&receipt.snapshot_identity.clone()));
    assert!(!receipt
        .commit_identity
        .is_same_current_identity_as(&copied_commit));
    assert!(!receipt
        .snapshot_identity
        .is_same_current_identity_as(&copied_snapshot));
}

#[test]
fn empty_runtime_snapshot_is_current_but_public_empty_projection_is_not() {
    let workspace =
        WorthQueryMemoryWorkspace::collection("Task", [aspect("identity.id", "identity.id")])
            .expect("memory workspace should build");
    let current = workspace
        .snapshot_identity()
        .expect("the fixture owner remains open");
    let copied_projection = WorthQuerySnapshotIdentity::empty_relational_state();

    assert!(current.is_same_current_identity_as(&current.clone()));
    assert!(!current.is_same_current_identity_as(&copied_projection));
}

#[test]
fn memory_workspace_update_and_delete_preserve_entity_lifecycle() {
    let mut workspace = WorthQueryMemoryWorkspace::collection(
        "Task",
        [
            aspect("identity.id", "identity.id"),
            aspect("title.value", "title.value"),
        ],
    )
    .expect("memory workspace should build");

    let insert = workspace
        .insert_aspects(vec![
            WorthQueryAuthoredAspectMutation::new(touch("identity.id"), text("task-1"))
                .expect("identity aspect"),
            WorthQueryAuthoredAspectMutation::new(touch("title.value"), text("First task"))
                .expect("title aspect"),
        ])
        .expect("seed insert should succeed");
    let entity_identity = insert.deltas[0].entity_identity.clone();

    let update = workspace
        .update_aspects(
            entity_identity.clone(),
            vec![
                WorthQueryAuthoredAspectMutation::new(touch("title.value"), text("Updated task"))
                    .expect("title aspect"),
            ],
        )
        .expect("update should succeed");
    assert_eq!(update.deltas[0].kind, WorthQueryMutationKind::Updated);
    assert_eq!(
        update.deltas[0].admitted_touched_aspects(),
        &[touch("title.value")]
    );
    assert_eq!(
        workspace.entities().unwrap()[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Updated task"))
    );
    assert_eq!(
        workspace.entities().unwrap()[0].aspect_value(
            &worth_foundational::facade::AspectKey::new("title")
                .expect("title should be an aspect key"),
        ),
        Some(&text("Updated task"))
    );

    let delete = workspace
        .delete(entity_identity)
        .expect("delete should succeed");
    assert_eq!(delete.deltas[0].kind, WorthQueryMutationKind::Deleted);
    assert!(workspace.entities().unwrap().is_empty());
}

#[test]
fn memory_workspace_matches_declared_aspects_with_native_touches() {
    let mut workspace =
        WorthQueryMemoryWorkspace::collection("Task", [aspect("title", "title.value")])
            .expect("memory workspace should build");

    workspace
        .insert_aspects(vec![WorthQueryAuthoredAspectMutation::new(
            touch("title.value"),
            text("Native match"),
        )
        .expect("title field touch")])
        .expect("field touch should match whole-aspect declaration natively");

    assert_eq!(
        workspace.entities().unwrap()[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Native match"))
    );
    assert_eq!(
        workspace.entities().unwrap()[0].aspect_value(
            &worth_foundational::facade::AspectKey::new("title")
                .expect("title should be an aspect key"),
        ),
        Some(&text("Native match"))
    );
}

#[test]
fn memory_workspace_aspect_rejects_mismatched_native_field_path() {
    let denial = crate::memory_workspace::WorthQueryAspect::new(
        touch("title.value"),
        field_path("identity.id"),
    )
    .expect_err("mismatched aspect touch and native field path should be denied");

    assert!(denial
        .message()
        .contains("must use native field path rooted at `title`"));
}

fn aspect(label: &str, native_field_path: &str) -> crate::memory_workspace::WorthQueryAspect {
    crate::memory_workspace::WorthQueryAspect::new(touch(label), field_path(native_field_path))
        .expect("test aspect should admit")
}

fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.')
            .map(FieldKey::new)
            .collect::<Option<Vec<_>>>()
            .expect("test field path segments should be valid"),
    )
    .expect("test field path should not be empty")
}

fn touch(touch_fixture: &str) -> crate::runtime::WorthQueryAspectTouch {
    let mut segments = touch_fixture.split('.');
    let aspect_key = AspectKey::new(
        segments
            .next()
            .expect("test touch fixture should name an aspect"),
    )
    .expect("test aspect key should admit");
    let field_segments = segments
        .map(|field| FieldKey::new(field).expect("test field key should admit"))
        .collect::<Vec<_>>();
    if field_segments.is_empty() {
        crate::runtime::WorthQueryAspectTouch::whole_aspect(aspect_key)
    } else {
        crate::runtime::WorthQueryAspectTouch::aspect_field_path(
            aspect_key,
            CanonicalFieldPath::new(field_segments).expect("test field path should admit"),
        )
    }
}

fn text(value: impl Into<String>) -> AspectValue {
    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value)
}

fn optional_string_contract(name: &str) -> AspectContract {
    aspects()
        .contract()
        .for_key(AspectKey::new(name).unwrap())
        .identified_by(AspectIdentity(91))
        .at_revision(AspectContractRevision(1))
        .scalar_with(
            ScalarAspectType::String,
            AspectMaskContract::scalar(),
            AbsenceLaw::Optional,
            AspectEquivalenceBasis::ExactCanonicalValue,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
}

fn summary_contract() -> AspectContract {
    let fields = StructAspectShape::new([
        struct_field("title", FieldRequirement::Required, AbsenceLaw::Required),
        struct_field("status", FieldRequirement::Optional, AbsenceLaw::Optional),
    ])
    .unwrap();
    aspects()
        .contract()
        .for_key(AspectKey::new("summary").unwrap())
        .identified_by(AspectIdentity(92))
        .at_revision(AspectContractRevision(1))
        .struct_aspect(fields)
}

fn struct_field(
    name: &str,
    requirement: FieldRequirement,
    absence: AbsenceLaw,
) -> FieldDeclaration {
    FieldDeclaration::new(
        FieldKey::new(name).unwrap(),
        ScalarAspectType::String,
        requirement,
        absence,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap()
}
