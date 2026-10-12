//! Law 1, reach: these fault doors are sealed by the control-file boundary.
use super::authoring::WorthQueryCheckpointMigrationWriter;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryCheckpointMigrationWriter<'_, Schema> {
    #[doc(hidden)]
    pub fn fail_next_durable_append_for_test(&self) {
        self.graph
            .graph
            .integration_handle()
            .with_runtime(|runtime| runtime.fail_next_durable_append_for_test());
    }
}
