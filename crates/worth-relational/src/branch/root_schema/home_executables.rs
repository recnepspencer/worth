//! Receiving frozen codecs bind without replacing registered Source issuers.
use super::*;
use crate::branch::RelationalBranchRootCaptureDenial as Denial;
impl RelationalBranchRootSchemaAuthority {
    pub(crate) fn bind_home_executables(
        self: &Arc<Self>,
        available: &crate::content::RelationalContentFormatRegistry,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let control = crate::schema::SchemaCompilationControl::Request(*request);
        control.checkpoint().map_err(Denial::Request)?;
        if self.registry.content_formats.is_bound_to(available) {
            return Ok(self.clone());
        }
        let quote =
            compilation_quote::SchemaCompilationQuote::for_target(&self.registry, self, control)?;
        let (memory, temporary) = quote.reserve(request)?;
        let mut registry = self.registry.as_ref().clone();
        registry.content_formats = registry
            .content_formats
            .bind_recorded(available)
            .map_err(Denial::Content)?;
        let expected = self
            .registry
            .snapshot_for_compilation(control)
            .map_err(Denial::Request)?;
        // Registry cloning preserves the actual registered recipe/domain Arcs,
        // including Source program bindings installed by the shared compiler.
        let bound = Self::compile_owned(
            self.allocation_id,
            registry,
            &expected,
            self.descriptor_semantics_version,
            compilation::RetainedContracts::Extend(Some(self)),
            Some(Arc::new(compiled_retention::SchemaCompilationFunding {
                _memory: memory,
            })),
            control,
        )
        .map_err(Denial::Request)?;
        if let Some(image) = self.recorded_image.get() {
            // A descriptive image has no executable format implementation.
            let _ = bound.recorded_image.set(image.clone());
        }
        drop(expected);
        drop(temporary);
        Ok(bound)
    }
}
