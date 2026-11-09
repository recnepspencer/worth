//! Actual owned maps and copied keys of a content-registry clone.
use super::RelationalContentFormatRegistry;
use crate::execution::RelationalExecutionDenialCause as Cause;
pub(crate) struct ContentRegistryCopyGeometry {
    pub(crate) formats: usize,
    pub(crate) names: usize,
    pub(crate) bindings: usize,
    pub(crate) version_node_bound: usize,
    pub(crate) name_bytes: usize,
}
impl RelationalContentFormatRegistry {
    pub(crate) fn copy_geometry(
        &self,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<ContentRegistryCopyGeometry, Cause> {
        let mut geometry = ContentRegistryCopyGeometry {
            formats: self.formats.len(),
            names: self.names.len(),
            bindings: self.bindings.len(),
            version_node_bound: 0,
            name_bytes: 0,
        };
        for (name, versions) in &self.names {
            control.checkpoint()?;
            geometry.name_bytes = geometry
                .name_bytes
                .checked_add(name.len())
                .ok_or(Cause::MemoryOverflow)?;
            if !versions.is_empty() {
                let nodes = versions.len().checked_add(1).ok_or(Cause::MemoryOverflow)?;
                geometry.version_node_bound = geometry
                    .version_node_bound
                    .checked_add(nodes)
                    .ok_or(Cause::MemoryOverflow)?;
            }
        }
        Ok(geometry)
    }
}
