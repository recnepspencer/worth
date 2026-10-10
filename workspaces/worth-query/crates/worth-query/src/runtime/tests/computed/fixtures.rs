use super::*;

pub(super) fn external_commit(label: &str) -> WorthQueryCommitIdentity {
    crate::memory_workspace::admit_external_commit_label(label)
}

pub(super) fn touch(path: &str) -> WorthQueryAspectTouch {
    test_aspect_touch(path)
}

pub(super) fn touches<const N: usize>(paths: [&str; N]) -> [WorthQueryAspectTouch; N] {
    paths.map(test_aspect_touch)
}

pub(super) fn update_string_aspect(
    entity_identity: crate::memory_workspace::WorthQueryEntityIdentity,
    authored_touch_text: &str,
    value: &str,
) -> WorthQueryWriteCommand {
    WorthQueryWriteCommand::UpdateAspect {
        entity_identity,
        aspect: WorthQueryAuthoredAspectMutation::new_set(
            touch(authored_touch_text),
            test_string_aspect_value(value),
        )
        .expect("test aspect update should build"),
    }
}

pub(super) fn read_derived_value_aspects<T>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryRuntime,
    view: &WorthQueryDerivedViewHandle<T>,
) -> Vec<AspectValue> {
    retained_value_aspects(read_derived(request_execution, runtime, view).retained_rows())
}

pub(super) fn read_derived<T>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryRuntime,
    view: &WorthQueryDerivedViewHandle<T>,
) -> WorthQueryDerivedMaterializationResult {
    runtime
        .read_derived_result(request_execution, view)
        .expect("test derived materialization should execute")
}

pub(super) fn retained_value_aspects(
    rows: &[WorthQueryRetainedMaterializedRow],
) -> Vec<AspectValue> {
    let value_path =
        retained_test_field_path("value").expect("test retained value path should parse");
    rows.iter()
        .filter_map(|row| row.scalar_value_at(&value_path))
        .cloned()
        .collect()
}

pub(super) fn retained_string_field(
    row: &WorthQueryRetainedMaterializedRow,
    field: &str,
) -> String {
    let field_path =
        retained_test_field_path(field).expect("test retained string path should parse");
    let value = row
        .scalar_value_at(&field_path)
        .expect("retained row should carry requested string field");
    let AspectValue::String(worth_foundational::facade::InternedString::Raw(value)) = value else {
        panic!("expected retained string field `{field}`, got {value:?}");
    };
    value.clone()
}

pub(super) fn retained_u64_field(row: &WorthQueryRetainedMaterializedRow, field: &str) -> u64 {
    let field_path =
        retained_test_field_path(field).expect("test retained integer path should parse");
    let value = row
        .scalar_value_at(&field_path)
        .expect("retained row should carry requested integer field");
    let AspectValue::UInt64(value) = value else {
        panic!("expected retained u64 field `{field}`, got {value:?}");
    };
    *value
}

pub(super) fn delta_or_produced_touches(
    view: &WorthQueryDerivedView,
    delta: &crate::memory_workspace::WorthQueryMutationDelta,
) -> Vec<WorthQueryAspectTouch> {
    if view.produced_aspect_touches().is_empty() {
        delta.admitted_touched_aspects().to_vec()
    } else {
        view.produced_aspect_touches().to_vec()
    }
}

pub(super) fn dependency_or_produced_touches(
    view: &WorthQueryDerivedView,
) -> Vec<WorthQueryAspectTouch> {
    if view.produced_aspect_touches().is_empty() {
        view.dependency_aspect_touches().to_vec()
    } else {
        view.produced_aspect_touches().to_vec()
    }
}
