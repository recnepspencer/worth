//! preview observation.
use super::*;

impl ConcurrentMatrixState {
    pub(super) fn record_preview_discard(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        let discarded = {
            let mut preview = self
                .workspace
                .preview(
                    request_execution,
                    test_session_label("phase16-preview-discard"),
                )
                .expect("preview should admit");
            preview
                .insert(request_execution, "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("phase16-preview-discard"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview Discard"),
                    )
                })
                .expect("preview insert should stage");
            preview.discard()
        };
        self.delivery_residue_count += hostile_preview_delivery_residue_count(&discarded);
        self.preview_closeout_digests
            .push(hostile_preview_closeout_digest(&discarded));
    }

    pub(super) fn record_preview_promotion(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        let promoted = {
            let mut preview = self
                .workspace
                .preview(
                    request_execution,
                    test_session_label("phase16-preview-promote"),
                )
                .expect("preview should admit");
            preview
                .insert(request_execution, "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("phase16-preview-promote"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview Promote"),
                    )
                })
                .expect("preview insert should stage");
            preview
                .promote(request_execution)
                .expect("preview should promote")
        };
        self.delivery_residue_count += hostile_preview_delivery_residue_count(&promoted);
        self.preview_closeout_digests
            .push(hostile_preview_closeout_digest(&promoted));
    }
}
