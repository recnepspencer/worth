use std::sync::Arc;

use crate::runtime::tests::support::*;
use crate::runtime::{
    WorthQueryDerivedPatch, WorthQueryDerivedPatchPayload, WorthQueryDerivedViewMaintainer,
};

#[derive(Clone)]
pub(super) struct ConcurrentMatrixMaintainer {
    titles: Arc<Vec<String>>,
}

impl ConcurrentMatrixMaintainer {
    pub(super) fn seeded() -> Self {
        Self {
            titles: Arc::new(vec!["Task One".to_string()]),
        }
    }
}

impl WorthQueryDerivedViewMaintainer for ConcurrentMatrixMaintainer {
    fn maintain(
        &mut self,
        view: &crate::program::WorthQueryDerivedView,
        _delta: &crate::memory_workspace::WorthQueryMutationDelta,
        materialization: &mut WorthQueryDerivedViewMaterialization,
    ) -> WorthQueryDerivedPatch {
        let title = self
            .titles
            .last()
            .cloned()
            .unwrap_or_else(|| "Unpublished".to_string());
        let retained_row = retained_string_test_row("title.value", title.clone());
        materialization.replace_retained_rows([retained_row.clone()]);
        WorthQueryDerivedPatch::whole_refresh_materialized(
            view.name(),
            crate::memory_workspace::admit_external_commit_label(format!(
                "phase16-publication-{}",
                title
            ))
            .expect("fixture digest must fit"),
            [test_aspect_touch("title.value")],
            WorthQueryDerivedPatchPayload::from_retained_row(retained_row),
            format!("phase16-publication-{}", title),
        )
    }
}
