use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_foundational::facade::AspectValue;

use crate::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath, RawAuthoredQuery,
    RawAuthoredResultShape, RootEntityKey,
};
use crate::authorized_projection::{
    derive_authorized_projection, AuthorizedProjectionArtifact, PolicyAspectMask,
    PolicyInfluenceSet,
};
use crate::canonicalization::CanonicalResultShapeArtifact;
use crate::projection_consumption::{ProjectionAuthorityContract, ProjectionAuthorityOutcome};

use super::support::*;

#[derive(Clone)]
pub(super) struct SharedReadPublishingMaintainer {
    pub(super) invocations: Arc<AtomicUsize>,
    pub(super) mode: SharedReadPublicationMode,
}

#[derive(Clone)]
pub(super) enum SharedReadPublicationMode {
    RefreshTitle(&'static str),
    EmptyRefresh,
    SequencedRefresh(&'static [&'static str]),
    IncrementalTitle(&'static str),
}

impl WorthQueryDerivedViewMaintainer for SharedReadPublishingMaintainer {
    fn maintain(
        &mut self,
        view: &crate::program::WorthQueryDerivedView,
        _delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let next = self.invocations.fetch_add(1, Ordering::SeqCst) + 1;
        match self.mode {
            SharedReadPublicationMode::RefreshTitle(title) => {
                materialization.replace_retained_rows([published_title_retained_row(title)]);
                WorthQueryDerivedPatch::whole_refresh_materialized(
                    view.name(),
                    crate::memory_workspace::admit_external_commit_label(format!(
                        "shared-read-refresh-{next}"
                    )),
                    test_aspect_touches(["title.value"]),
                    WorthQueryDerivedPatchPayload::from_retained_row(published_title_retained_row(
                        title,
                    )),
                    format!("shared-read-publication-{next}"),
                )
            }
            SharedReadPublicationMode::EmptyRefresh => {
                materialization.replace_retained_rows(std::iter::empty());
                WorthQueryDerivedPatch::whole_refresh_materialized(
                    view.name(),
                    crate::memory_workspace::admit_external_commit_label(format!(
                        "shared-read-empty-{next}"
                    )),
                    test_aspect_touches(["title.value"]),
                    WorthQueryDerivedPatchPayload::empty(),
                    format!("shared-read-empty-publication-{next}"),
                )
            }
            SharedReadPublicationMode::SequencedRefresh(titles) => {
                let title = titles
                    .get(next.saturating_sub(1))
                    .copied()
                    .unwrap_or_else(|| titles[titles.len() - 1]);
                materialization.replace_retained_rows([published_title_retained_row(title)]);
                WorthQueryDerivedPatch::whole_refresh_materialized(
                    view.name(),
                    crate::memory_workspace::admit_external_commit_label(format!(
                        "shared-read-sequenced-{next}"
                    )),
                    test_aspect_touches(["title.value"]),
                    WorthQueryDerivedPatchPayload::from_retained_row(published_title_retained_row(
                        title,
                    )),
                    format!("shared-read-sequenced-publication-{next}"),
                )
            }
            SharedReadPublicationMode::IncrementalTitle(title) => {
                materialization.replace_retained_rows([published_title_retained_row(title)]);
                WorthQueryDerivedPatch::incremental(
                    view.name(),
                    crate::memory_workspace::admit_external_commit_label(format!(
                        "shared-read-stale-{next}"
                    )),
                    crate::memory_workspace::admit_authored_entity_label(format!("entity-{next}")),
                    test_aspect_touches(["title.value"]),
                    WorthQueryDerivedPatchPayload::from_retained_row(published_title_retained_row(
                        title,
                    )),
                )
            }
        }
    }
}

pub(super) fn shared_read_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> WorthQueryWorkspace {
    stateful_bridge_task_runtime(request_execution)
        .workspace(name)
        .expect("workspace should build")
}

pub(super) fn declare_shared_read_derived(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    suffix: &str,
    maintainer: SharedReadPublishingMaintainer,
) -> WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> {
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view_request(
            request_execution,
            format!("tasks.{suffix}"),
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    workspace
        .computed_view(
            request_execution,
            crate::program::WorthQueryDerivedView::new(
                format!("derived.{suffix}"),
                test_aspect_touches(["title.value"]),
            )
            .depends_on_live(&live),
            maintainer,
        )
        .expect("derived view should declare")
}

pub(super) fn insert_task(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    id: &str,
    title: &str,
) {
    workspace
        .insert(request_execution, "Task", |builder| {
            builder
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value(id),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value(title),
                )
        })
        .expect("task insert should succeed");
}

pub(super) fn consume_display_title_attempt(
    artifact: &WorthQueryPublishedDerivedArtifactHandle,
) -> WorthQueryPublishedProjectionAuthorityOutcome {
    let (result_shape, authorized_projection) = projection_artifacts();
    artifact
        .consume_projection_authority(
            &result_shape,
            &authorized_projection,
            ProjectionAuthorityContract::declare()
                .require_settled_consumption()
                .require_source_authority()
                .require_display_field(
                    crate::projection_consumption::projection_fact_field_path_from_segments([
                        worth_foundational::facade::FieldKey::new("title")
                            .expect("projection fact field segment should admit"),
                        worth_foundational::facade::FieldKey::new("value")
                            .expect("projection fact field segment should admit"),
                    ]),
                ),
        )
        .expect("projection consumption should stay on the typed artifact lane")
}

pub(super) fn consume_display_title(artifact: &WorthQueryPublishedDerivedArtifactHandle) -> String {
    let completed = match consume_display_title_attempt(artifact) {
        WorthQueryPublishedProjectionAuthorityOutcome::Current(
            ProjectionAuthorityOutcome::Admitted(authority),
        ) => authority,
        other => panic!("expected admitted published consumption, got {other:?}"),
    };
    completed
        .facts()
        .display_fields()
        .first()
        .and_then(|fact| match fact.native_value().scalar()? {
            AspectValue::String(worth_foundational::facade::InternedString::Raw(value)) => {
                Some(value.clone())
            }
            _ => None,
        })
        .expect("display-field title should be present")
}

fn projection_artifacts() -> (CanonicalResultShapeArtifact, AuthorizedProjectionArtifact) {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("task").unwrap())
        .project(AspectFieldSelector::new("title", "value").unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("title", "value", "title.value").unwrap())
        .build()
        .unwrap();
    let canonical = GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap();
    let authorized_projection = derive_authorized_projection(
        canonical.query(),
        canonical.result_shape(),
        "policy:test",
        "schema:test",
        &PolicyAspectMask::allow_all(),
        &PolicyInfluenceSet::none(),
        8,
        8,
    )
    .unwrap();
    (canonical.result_shape().clone(), authorized_projection)
}

fn published_title_retained_row(title: &str) -> WorthQueryRetainedMaterializedRow {
    retained_string_test_row("title.value", title)
}
