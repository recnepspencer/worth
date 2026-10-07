use super::{ApplicationHome, WorthQueryApplicationCloseRefusal};
use crate::domain_computation::primary_graph::{
    application_installation::WorthQueryProgramApplicationRuntime,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn close(self) -> Result<ApplicationHome, WorthQueryApplicationCloseRefusal<Self>> {
        let result = self
            .primary_provider
            .graph
            .source_owner
            .close_with_image(|native| {
                self.assemble_application_checkpoint(native)
                    .map(|(image, _)| image)
            });
        match result {
            Ok(image) => Ok(ApplicationHome::holding(image)),
            Err(denial) => Err(WorthQueryApplicationCloseRefusal {
                runtime: self,
                denial,
            }),
        }
    }
}

impl<Schema: ApplicationSchema + 'static, Program>
    WorthQueryProgramApplicationRuntime<Schema, Program>
{
    pub fn close(self) -> Result<ApplicationHome, WorthQueryApplicationCloseRefusal<Self>> {
        let Self {
            runtime,
            program,
            connection_types,
            root_graph_types,
            output_source_bindings,
            action_bindings,
            supported,
            opening,
        } = self;
        runtime
            .close()
            .map_err(|refusal| WorthQueryApplicationCloseRefusal {
                runtime: Self {
                    runtime: refusal.runtime,
                    program,
                    connection_types,
                    root_graph_types,
                    output_source_bindings,
                    action_bindings,
                    supported,
                    opening,
                },
                denial: refusal.denial,
            })
    }
}
