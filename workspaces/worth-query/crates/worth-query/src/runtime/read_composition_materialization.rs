use std::collections::BTreeMap;

use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::WorthQueryEntity;
use crate::query_context::QueryContextExecutionArtifact;
use crate::runtime::{
    WorthQueryLiveArtifactTarget, WorthQueryReadDenial, WorthQueryReadDenialKind,
    WorthQueryReadGraph, WorthQueryReadGraphFamily, WorthQueryRuntime,
};
use crate::schema_view::QuerySchemaView;
use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};

mod projection;

pub(in crate::runtime) use projection::project_rows_to_request;

use super::read_composition_row_selection::{
    materialize_collection_rows_from_request, materialize_detail_rows_from_request,
    synthetic_detail_rows_for_request,
};

pub(in crate::runtime) struct WorthQueryMaterializedReadRows {
    rows: Vec<WorthQueryEntity>,
    maintenance_source_rows: Vec<WorthQueryEntity>,
    records_examined_count: usize,
}

impl WorthQueryMaterializedReadRows {
    pub(in crate::runtime) fn into_parts(
        self,
    ) -> (Vec<WorthQueryEntity>, Vec<WorthQueryEntity>, usize) {
        (
            self.rows,
            self.maintenance_source_rows,
            self.records_examined_count,
        )
    }
}

pub(in crate::runtime) fn materialize_read_rows(
    runtime: &mut WorthQueryRuntime,
    read_graph: &WorthQueryReadGraph,
) -> Result<WorthQueryMaterializedReadRows, WorthQueryReadDenial> {
    let view_name = format!("runtime.read.materialized:{}", read_graph.digest());
    let target = WorthQueryLiveArtifactTarget::from_view_name(view_name.clone());
    let request = read_graph.declarative_request().clone();
    ensure_materialized_read_view(runtime, &target, &view_name, read_graph)?;
    let source_rows =
        runtime
            .backend
            .live_entities_for_target(&target)
            .map_err(|error| match error.kind() {
                crate::memory_workspace::WorthQueryWorkspaceErrorKind::Handle(denial) => {
                    denial.into()
                }
                _ => WorthQueryReadDenial::new(
                    WorthQueryReadDenialKind::ExecutionDenied,
                    error.to_string(),
                ),
            })?;
    let records_examined_count = source_rows.len();
    let maintenance_source_rows =
        select_source_rows_for_read_graph(read_graph, &request, &source_rows);
    let rows = project_rows_to_request(maintenance_source_rows.clone(), &request);
    Ok(WorthQueryMaterializedReadRows {
        rows,
        maintenance_source_rows,
        records_examined_count,
    })
}

pub(in crate::runtime) fn materialize_source_rows_for_read_graph(
    read_graph: &WorthQueryReadGraph,
    request: &DeclarativeLiveQueryRequest,
    source_rows: &[WorthQueryEntity],
) -> Vec<WorthQueryEntity> {
    let rows = select_source_rows_for_read_graph(read_graph, request, source_rows);
    project_rows_to_request(rows, request)
}

fn select_source_rows_for_read_graph(
    read_graph: &WorthQueryReadGraph,
    request: &DeclarativeLiveQueryRequest,
    source_rows: &[WorthQueryEntity],
) -> Vec<WorthQueryEntity> {
    match read_graph.family() {
        WorthQueryReadGraphFamily::Detail => {
            materialize_detail_rows_from_request(read_graph, request, source_rows)
                .unwrap_or_else(|| synthetic_detail_rows_for_request(request))
        }
        WorthQueryReadGraphFamily::Collection => {
            materialize_collection_rows_from_request(read_graph, request, source_rows)
        }
    }
}

pub(in crate::runtime) fn materialize_query_context_rows(
    context_execution: &QueryContextExecutionArtifact,
) -> Vec<WorthQueryEntity> {
    context_execution
        .rows()
        .iter()
        .enumerate()
        .map(|(index, row)| {
            WorthQueryEntity::from_native_field_values(
                crate::memory_workspace::admit_authored_entity_label(format!(
                    "query-context:{}:{}",
                    context_execution.family().as_str(),
                    index
                )),
                query_context_row_values(context_execution, row),
            )
        })
        .collect()
}

fn query_context_row_values(
    context_execution: &QueryContextExecutionArtifact,
    row: &str,
) -> BTreeMap<CanonicalFieldPath, AspectValue> {
    BTreeMap::from([
        (
            native_field_path("query_context.basis_digest"),
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                context_execution.basis_digest().to_string(),
            ),
        ),
        (
            native_field_path("query_context.query_digest"),
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                context_execution.query_digest().to_string(),
            ),
        ),
        (
            native_field_path("query_context.result_digest"),
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                context_execution.result_digest().to_string(),
            ),
        ),
        (
            native_field_path("query_context.row"),
            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(row.to_string()),
        ),
    ])
}

fn ensure_materialized_read_view(
    runtime: &mut WorthQueryRuntime,
    target: &WorthQueryLiveArtifactTarget,
    view_name: &str,
    read_graph: &WorthQueryReadGraph,
) -> Result<(), WorthQueryReadDenial> {
    let request = read_graph.declarative_request();
    ensure_materialized_read_target_available(&runtime.materialized_read_views, target, request)?;
    admit_materialized_read_view(runtime, view_name, request, read_graph.schema_view())?;
    declare_materialized_read_view(
        runtime,
        target,
        view_name,
        request,
        read_graph.schema_view(),
    )
}

fn ensure_materialized_read_target_available(
    materialized_read_views: &BTreeMap<WorthQueryLiveArtifactTarget, DeclarativeLiveQueryRequest>,
    target: &WorthQueryLiveArtifactTarget,
    request: &DeclarativeLiveQueryRequest,
) -> Result<(), WorthQueryReadDenial> {
    if let Some(existing) = materialized_read_views.get(target) {
        if existing != request {
            return Err(WorthQueryReadDenial::new(
                WorthQueryReadDenialKind::ExecutionDenied,
                "materialized read view name collision with mismatched declarative request",
            ));
        }
    }
    Ok(())
}

fn admit_materialized_read_view(
    runtime: &WorthQueryRuntime,
    view_name: &str,
    request: &DeclarativeLiveQueryRequest,
    schema_view: &QuerySchemaView,
) -> Result<(), WorthQueryReadDenial> {
    let admission_receipt = runtime
        .backend
        .admit_live_view_declaration(view_name, request, schema_view)
        .map_err(execution_denial_from_workspace_error)?;
    if let Some(message) = admission_receipt.drift_from_request(view_name, request) {
        return Err(WorthQueryReadDenial::new(
            WorthQueryReadDenialKind::ExecutionDenied,
            message,
        ));
    }
    Ok(())
}

fn declare_materialized_read_view(
    runtime: &mut WorthQueryRuntime,
    target: &WorthQueryLiveArtifactTarget,
    view_name: &str,
    request: &DeclarativeLiveQueryRequest,
    schema_view: &QuerySchemaView,
) -> Result<(), WorthQueryReadDenial> {
    if runtime.materialized_read_views.contains_key(target) {
        return Ok(());
    }
    runtime
        .backend
        .declare_live_view(view_name.to_string(), request.clone(), schema_view.clone())
        .map_err(execution_denial_from_workspace_error)?;
    runtime
        .materialized_read_views
        .insert(target.clone(), request.clone());
    Ok(())
}

fn execution_denial_from_workspace_error(
    error: crate::memory_workspace::WorthQueryWorkspaceError,
) -> WorthQueryReadDenial {
    WorthQueryReadDenial::new(WorthQueryReadDenialKind::ExecutionDenied, error.to_string())
}

fn native_field_path(path: impl AsRef<str>) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.as_ref()
            .split('.')
            .map(FieldKey::new)
            .collect::<Option<Vec<_>>>()
            .expect("query-context field path segments must be foundational"),
    )
    .expect("query-context field path must not be empty")
}
