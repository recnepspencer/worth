use crate::memory_workspace::WorthQueryWorkspaceErrorKind;
use crate::runtime::{
    InvariantCatalog, InvariantRegistration, InvariantRule, WorthQueryAspectTouch,
    WorthQueryInspection, WorthQueryMutationFamily, WorthQueryPreviewOptions,
    WorthQueryRuntimeError, WorthQueryUnrefinedLiveShape,
};
use crate::session_label::WorthQuerySessionLabel;
use std::num::NonZeroUsize;
use worth_foundational::facade::{AspectKey, AspectValue, CanonicalFieldPath, FieldKey};

use super::{in_memory_test_runtime, WorthQueryTestBackendSchema};

mod atomic_setup;

#[test]
fn in_memory_workspace_exposes_bounded_creation_recovery_and_cleanup_discovery() {
    let workspace = task_workspace();
    let page = workspace
        .branches()
        .recovery_page(None, NonZeroUsize::new(1).unwrap())
        .expect("the outer workspace must expose bounded World recovery discovery");
    assert!(page.rows().is_empty());
    assert!(workspace.branches().pending_cleanup().is_empty());
}

#[test]
fn in_memory_test_runtime_executes_public_insert_and_live_read() {
    let mut workspace = task_workspace();
    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>("consumer-kit.test.tasks", |view| {
            view.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
        })
        .expect("test backend should declare a live view for its collection");

    let receipt = workspace
        .insert("Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-1"))
                .set_aspect(touch("title.value"), authored_text("Write real tests"))
        })
        .expect("test backend should execute public workspace insert");

    assert_eq!(workspace.read(&tasks).len(), 1);
    assert_eq!(
        workspace
            .read_live_result(&tasks)
            .expect("typed live read should execute")
            .rows()
            .len(),
        1
    );
    match workspace
        .inspect(&receipt)
        .expect("write receipt should inspect")
    {
        WorthQueryInspection::WriteReceipt(inspection) => {
            assert_eq!(
                inspection.runtime_evidence().artifact_family(),
                "consumer-kit-in-memory-test-write-receipt"
            );
            assert_eq!(
                inspection.runtime_evidence().evidence(),
                &["consumer-kit-in-memory-test-inspection".to_string()]
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
    let preview_label =
        WorthQuerySessionLabel::scoped_strs("consumer-kit-test-backend", ["preview"])
            .expect("preview label");
    let preview = workspace
        .preview(preview_label.clone())
        .expect("test backend should admit preview basis");
    assert_eq!(preview.basis_admission().label(), preview_label.display());
    assert_eq!(
        preview.basis_admission().evidence(),
        vec!["consumer-kit-in-memory-test-backend".to_string()]
    );
}

#[test]
fn in_memory_test_runtime_executes_update_delete_and_live_routing() {
    let mut workspace = task_workspace();
    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>("consumer-kit.test.crud.tasks", |view| {
            view.from("Task").select([
                crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id").unwrap(),
                crate::authoring::AspectFieldKey::from_authoring_parts("title", "value").unwrap(),
            ])
        })
        .expect("task live view should declare");
    let insert = workspace
        .insert("Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-crud"))
                .set_aspect(touch("title.value"), authored_text("Draft"))
        })
        .expect("insert should execute");
    let entity_identity = insert
        .target_entity_identity()
        .expect("insert should expose target identity")
        .clone();

    let update = workspace
        .update(entity_identity.clone(), |task| {
            task.set_aspect(touch("title.value"), authored_text("Updated"))
        })
        .expect("update should execute");
    assert_eq!(update.mutation_family(), WorthQueryMutationFamily::Update);
    assert_eq!(
        update.terminal_affected_live_view_ids_projection(),
        &["consumer-kit.test.crud.tasks".to_string()]
    );
    assert_eq!(
        workspace.read(&tasks)[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Updated"))
    );

    let delete = workspace
        .delete(entity_identity)
        .expect("delete should execute");
    assert_eq!(delete.mutation_family(), WorthQueryMutationFamily::Delete);
    assert_eq!(
        delete.terminal_affected_live_view_ids_projection(),
        &["consumer-kit.test.crud.tasks".to_string()]
    );
    assert!(workspace.read(&tasks).is_empty());
}

#[test]
fn in_memory_test_runtime_stages_sandboxed_preview_writes_without_authoritative_residue() {
    let mut workspace = task_workspace();
    let preview_label =
        WorthQuerySessionLabel::scoped_strs("consumer-kit-test-backend", ["sandboxed-preview"])
            .expect("preview label");
    let mut preview = workspace
        .preview_with_options(
            preview_label,
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("sandboxed preview should admit");

    preview
        .insert("Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("preview-task"))
                .set_aspect(touch("title.value"), authored_text("Preview only"))
        })
        .expect("sandboxed preview write should stage");
    let outcome = preview.discard();

    assert!(outcome.discarded());
    assert_eq!(outcome.closeout_evidence().preview_write_staging_count(), 1);
    assert_eq!(outcome.write_count(), 1);
    assert_eq!(outcome.authoritative_residue_count(), 0);
    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>("consumer-kit.test.after-preview", |view| {
            view.from("Task").select([
                crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id").unwrap(),
                crate::authoring::AspectFieldKey::from_authoring_parts("title", "value").unwrap(),
            ])
        })
        .expect("live view should declare after preview discard");
    assert!(workspace.read(&tasks).is_empty());
}

#[test]
fn in_memory_test_runtime_denies_wrong_collection_preview_before_residue() {
    let mut workspace = task_workspace();
    let preview_label =
        WorthQuerySessionLabel::scoped_strs("consumer-kit-test-backend", ["bad-preview"])
            .expect("preview label");
    let mut preview = workspace
        .preview_with_options(
            preview_label,
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("sandboxed preview should admit");

    let error = preview
        .insert("Issue", |issue| {
            issue.set_aspect(touch("identity.id"), authored_text("issue-preview"))
        })
        .expect_err("preview write should honor backend schema before staging");
    assert_workspace_error_kind(error, WorthQueryWorkspaceErrorKind::UnsupportedCollection);
    let outcome = preview.discard();
    assert_eq!(outcome.closeout_evidence().preview_write_staging_count(), 0);
    assert_eq!(outcome.write_count(), 0);
    assert_eq!(outcome.authoritative_residue_count(), 0);
}

#[test]
fn in_memory_test_runtime_fails_closed_for_unsupported_collections() {
    let mut workspace = task_workspace();
    let error = workspace
        .insert("Issue", |issue| {
            issue
                .set_aspect(touch("identity.id"), authored_text("issue-1"))
                .set_aspect(touch("title.value"), authored_text("Wrong family"))
        })
        .expect_err("test backend should reject collections outside its schema");

    assert_workspace_error_kind(error, WorthQueryWorkspaceErrorKind::UnsupportedCollection);
}

#[test]
fn in_memory_test_runtime_lowers_invariant_catalog_into_real_write_denial() {
    let schema = task_schema();
    let mut workspace = in_memory_test_runtime()
        .with_schema(schema)
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::MaxMergedIntents(0),
            )],
        })
        .workspace("consumer-kit.test-backend.invariant-denial")
        .expect("in-memory test runtime with invariant catalog should build");

    let error = workspace
        .insert("Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-denied"))
                .set_aspect(touch("title.value"), authored_text("Denied by invariant"))
        })
        .expect_err("registered invariant should deny the write");

    assert_workspace_error_kind(error, WorthQueryWorkspaceErrorKind::Unclassified);
}

#[test]
fn in_memory_test_runtime_merges_repeated_invariant_catalog_inputs() {
    let schema = task_schema();
    let blocking_catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::MaxMergedIntents(0),
        )],
    };
    let harmless_catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::snapshot_publication_blocking(
            InvariantRule::MaxSnapshotEntities(99),
        )],
    };
    let mut workspace = in_memory_test_runtime()
        .with_schema(schema)
        .invariant_catalog(blocking_catalog)
        .invariant_catalog(harmless_catalog)
        .workspace("consumer-kit.test-backend.invariant-merge")
        .expect("in-memory test runtime with merged invariant catalogs should build");

    let error = workspace
        .insert("Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-denied"))
                .set_aspect(
                    touch("title.value"),
                    authored_text("Denied by merged invariant"),
                )
        })
        .expect_err("first invariant catalog must not be overwritten by the second");

    assert_workspace_error_kind(error, WorthQueryWorkspaceErrorKind::Unclassified);
}

fn task_workspace() -> crate::runtime::WorthQueryWorkspace {
    let schema = task_schema();
    in_memory_test_runtime()
        .with_schema(schema)
        .workspace("consumer-kit.test-backend")
        .expect("in-memory test runtime should build")
}

fn task_schema() -> WorthQueryTestBackendSchema {
    super::contract_fixtures::task_schema()
}

fn assert_workspace_error_kind(error: WorthQueryRuntimeError, kind: WorthQueryWorkspaceErrorKind) {
    match error {
        WorthQueryRuntimeError::Workspace(workspace_error) => {
            assert_eq!(workspace_error.kind(), kind, "{workspace_error}");
        }
        other => panic!("expected workspace error `{kind:?}`, got {other:?}"),
    }
}

fn touch(touch_fixture: &str) -> WorthQueryAspectTouch {
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
        WorthQueryAspectTouch::whole_aspect(aspect_key)
    } else {
        WorthQueryAspectTouch::aspect_field_path(
            aspect_key,
            CanonicalFieldPath::new(field_segments).expect("test field path should admit"),
        )
    }
}

fn authored_text(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn text(value: impl Into<String>) -> AspectValue {
    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value)
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
